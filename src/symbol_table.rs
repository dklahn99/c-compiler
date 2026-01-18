use crate::ast::*;
use std::collections::HashMap;

// TODO: each variable should have a unique ID so that the get() method can be more specific
#[derive(Debug)]
pub struct SymbolTable {
    vars: HashMap<(u32, String), VarInfo>, // key is (scope_id, var_name)
    scope_tree: HashMap<u32, u32>,         // maps scope id to parent scope id
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            vars: HashMap::new(),
            scope_tree: HashMap::new(),
        }
    }

    pub fn from_function(dec: &Declaration) -> Self {
        // TODO: also add args to scope
        let Declaration::Function { scope, .. } = dec;
        Self::from_scope(scope)
    }

    fn from_scope(scope: &Scope) -> Self {
        let Scope { id, statements } = scope;

        let mut table = Self::new();

        for s in statements {
            match s {
                Statement::VarDeclare { name, var_type, .. } => {
                    table.vars.insert(
                        (*id, name.clone()),
                        VarInfo {
                            name: name.clone(),
                            var_type: var_type.clone(),
                        },
                    );
                }
                Statement::If {
                    true_block,
                    false_block,
                    ..
                } => {
                    table.add_child_scope(*id, true_block);
                    if false_block.is_some() {
                        let false_scope = false_block.as_ref().unwrap();
                        table.add_child_scope(*id, false_scope);
                    }
                }
                _ => {}
            }
        }

        table
    }

    fn merge(&mut self, other: SymbolTable) {
        self.vars.extend(other.vars);
        self.scope_tree.extend(other.scope_tree);
    }

    fn add_child_scope(&mut self, parent_id: u32, child: &Scope) {
        let child_table = Self::from_scope(child);
        self.merge(child_table);
        self.scope_tree.insert(child.id, parent_id);
    }

    pub fn get(&self, scope_id: u32, var_name: &String) -> Option<&VarInfo> {
        // look up starting scope with scope_id
        // search ScopeTable and parents for var_name
        match self.vars.get(&(scope_id, var_name.clone())) {
            Some(x) => Some(x),
            None => {
                let parent_scope = self.scope_tree.get(&scope_id);
                match parent_scope {
                    Some(parent_id) => self.get(*parent_id, var_name),
                    None => None,
                }
            }
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_st_debug() -> Result<(), String> {
        let scope = Scope {
            id: 1,
            statements: vec![
                Statement::VarDeclare {
                    name: "x".to_owned(),
                    var_type: Type::Int,
                    value: None,
                },
                Statement::If {
                    condition: Expr::IntLiteral(1),
                    true_block: Scope {
                        id: 2,
                        statements: vec![Statement::VarDeclare {
                            name: "x".to_owned(),
                            var_type: Type::Int,
                            value: None,
                        }],
                    },
                    false_block: Some(Scope {
                        id: 3,
                        statements: vec![Statement::VarDeclare {
                            name: "y".to_owned(),
                            var_type: Type::Int,
                            value: None,
                        }],
                    }),
                },
            ],
        };
        let st = SymbolTable::from_scope(&scope);
        println!("{:?}", st);
        println!("{:?}", st.get(1, &"x".to_owned()));
        println!("{:?}", st.get(2, &"x".to_owned()));
        println!("{:?}", st.get(3, &"x".to_owned()));
        println!("{:?}", st.get(3, &"y".to_owned()));
        println!("{:?}", st.get(1, &"y".to_owned()));
        Ok(())
    }
}
