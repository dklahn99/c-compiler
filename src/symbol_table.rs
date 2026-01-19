use crate::ast::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct SymbolTable {
    vars: HashMap<(u32, String), VarInfo>, // key is (scope_id, var_name)
    scope_tree: HashMap<u32, u32>,         // maps scope id to parent scope id
}

impl SymbolTable {
    fn new() -> Self {
        SymbolTable {
            vars: HashMap::new(),
            scope_tree: HashMap::new(),
        }
    }

    pub fn from_function(dec: &Declaration) -> Result<Self, String> {
        // TODO: also add args to scope
        let Declaration::Function { scope, .. } = dec;
        Self::from_scope(scope)
    }

    fn from_scope(scope: &Scope) -> Result<Self, String> {
        let Scope { id, statements } = scope;

        let mut table = Self::new();

        for s in statements {
            match s {
                Statement::VarDeclare { name, var_type, .. } => table.insert(
                    *id,
                    name,
                    VarInfo {
                        name: name.clone(),
                        var_type: var_type.clone(),
                    },
                )?,
                Statement::If {
                    true_block,
                    false_block,
                    ..
                } => {
                    table.add_child_scope(*id, true_block);
                    if let Some(false_scope) = false_block {
                        table.add_child_scope(*id, false_scope);
                    }
                }
                _ => {}
            }
        }

        Ok(table)
    }

    fn insert(&mut self, scope_id: u32, var_name: &str, var_info: VarInfo) -> Result<(), String> {
        if let Some(_) = self.get(scope_id, var_name) {
            return Err(format!(
                "Duplicate insertion of variable {:} into scope {:}.",
                var_name, scope_id
            ));
        }
        self.vars.insert((scope_id, var_name.to_owned()), var_info);
        Ok(())
    }

    fn merge(&mut self, other: SymbolTable) {
        self.vars.extend(other.vars);
        self.scope_tree.extend(other.scope_tree);
    }

    fn add_child_scope(&mut self, parent_id: u32, child: &Scope) -> Result<(), String> {
        let child_table = Self::from_scope(child)?;
        self.merge(child_table);
        self.scope_tree.insert(child.id, parent_id);
        Ok(())
    }

    pub fn get(&self, scope_id: u32, var_name: &str) -> Option<&VarInfo> {
        // If current scope has the variable, return it.
        // Otherwise, search the parent scope.
        if let Some(var_info) = self.vars.get(&(scope_id, var_name.to_owned())) {
            return Some(var_info);
        }
        if let Some(parent_scope) = self.scope_tree.get(&scope_id) {
            return self.get(*parent_scope, var_name);
        }
        None
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_st() -> Result<(), String> {
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
                            var_type: Type::UserDefined("MyType".to_owned()),
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
        let st = SymbolTable::from_scope(&scope)?;
        assert_eq!(
            st.get(1, "x"),
            Some(&VarInfo {
                name: "x".to_owned(),
                var_type: Type::Int
            })
        );
        assert_eq!(
            st.get(2, "x"),
            Some(&VarInfo {
                name: "x".to_owned(),
                var_type: Type::UserDefined("MyType".to_owned())
            })
        );
        assert_eq!(
            st.get(3, "x"),
            Some(&VarInfo {
                name: "x".to_owned(),
                var_type: Type::Int
            })
        );
        assert_eq!(
            st.get(3, "y"),
            Some(&VarInfo {
                name: "y".to_owned(),
                var_type: Type::Int
            })
        );
        assert_eq!(st.get(2, "y"), None);
        Ok(())
    }
}
