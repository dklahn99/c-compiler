use crate::ast;
use crate::symantic_check;
use crate::symbol_table::SymbolTable;
use std::collections::HashMap;

// Defines the Control Flow GRaph types
/*
  - something to generate variable names/labels
  SSA three address code:
  v1 = 5
  v2 = 10
  v3 = v1 + v2
  HashMap<id, ControlBlock>
  Statements:
    - if var then goto A else goto B
    - goto
    - unary assign
    - binary operations
    - return var
*/
type VarName = String;
type ControlBlockId = u64;

#[derive(Debug, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    // TODO: add in conditional support later
    // If {
    //     var: VarName,
    //     goto_true: ControlBlockId,
    //     goto_false: ControlBlockId,
    // },
    Goto(ControlBlockId),
    Assign {
        var: VarName,
        value: u64,
    },
    Operation {
        dest: VarName,
        op: BinOp,
        lhs: VarName,
        rhs: VarName,
    },
    Return(VarName),
}

struct CFGBuildContext {
    var_counter: u64,
    var_map: HashMap<String, String>, // maps Symbol Table var names to CFG var names (e.g. "x" -> "v1")
}

impl CFGBuildContext {
    fn new() -> Self {
        CFGBuildContext {
            var_counter: 0,
            var_map: HashMap::new(),
        }
    }

    fn inc(&mut self) -> VarName {
        self.var_counter += 1;
        format!("v{:}", self.var_counter)
    }

    fn register_var(&mut self, var: VarName) {
        let a = self.inc();
        self.var_map.insert(var, a);
    }

    fn lookup(&self, var: &VarName) -> Option<&VarName> {
        self.var_map.get(var)
    }
}

type ControlBlock = Vec<Statement>;
struct ControlFlowGraph(HashMap<ControlBlockId, ControlBlock>);

impl ControlFlowGraph {
    fn new() {}

    pub fn from(declarations: &Vec<ast::Declaration>, symbol_table: &SymbolTable) -> Self {
        // For now, we're only considering programs with a single declaration: a main function
        assert_eq!(declarations.len(), 1);

        let ast::Declaration::Function {
            name,
            args,
            return_type,
            scope,
        } = &declarations[0];
        assert_eq!(name, "main");
        assert_eq!(args.len(), 0);
        assert_eq!(*return_type, ast::Type::Int);

        let mut context = CFGBuildContext::new();

        let mut block: ControlBlock = vec![];
        for stmt in &scope.statements {
            block.append(&mut ControlFlowGraph::process(stmt, &mut context).expect(""));
        }

        // build CFG. This should be a single block since we don't have any conditionals
        ControlFlowGraph(HashMap::from([(0, block)]))
    }

    fn process(
        stmt: &ast::Statement,
        context: &mut CFGBuildContext,
    ) -> Result<Vec<Statement>, String> {
        match stmt {
            ast::Statement::VarDeclare { .. } => {
                ControlFlowGraph::stmt_from_var_declare(&stmt, context)
            }
            _ => Err("Not Implemented".to_owned()),
        }
    }

    fn stmt_from_var_declare(
        stmt: &ast::Statement,
        context: &mut CFGBuildContext,
    ) -> Result<Vec<Statement>, String> {
        if let ast::Statement::VarDeclare {
            name,
            var_type,
            value,
        } = stmt
        {
            context.register_var(name.clone());
            let cfg_var_name = context.lookup(name).expect("");

            let unwrapped = value.as_ref().unwrap_or(&ast::Expr::IntLiteral(0));
            // TODO: process inner expression. For now, assume it's an int literal
            if let ast::Expr::IntLiteral(v) = unwrapped {
                return Ok(vec![Statement::Assign {
                    var: cfg_var_name.clone(),
                    value: *v,
                }]);
            }
            return Err(format!("Expected an IntLiteral, but got {:?}", value));
        }

        Err(format!("Expected a VarDeclare, but got {:?}", stmt))
    }

    fn var_name(i: u64) -> String {
        return format!("v{:}", i);
    }
}

mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::symantic_check::check_syntax;
    use crate::tokenizer::tokenize;
    use std::fs::read_to_string;

    #[test]
    fn test_cfg_var_declare() -> Result<(), String> {
        let vd = ast::Statement::VarDeclare {
            name: "x".to_owned(),
            var_type: ast::Type::Int,
            value: Some(ast::Expr::IntLiteral(123)),
        };

        let expected = vec![Statement::Assign {
            var: "v1".to_owned(),
            value: 123,
        }];

        let mut context = CFGBuildContext::new();
        assert_eq!(ControlFlowGraph::process(&vd, &mut context)?, expected);

        Ok(())
    }

    // #[test]
    // fn my_test() -> Result<(), String> {
    //     let s = read_to_string("test/main.c").unwrap();
    //     let tokens = tokenize(&s)?;
    //     let ast = parse(&tokens)?;
    //     let symbol_table = check_syntax(&ast)?;
    //     let cfg = ControlFlowGraph::from(&ast, &symbol_table);

    //     let control_block = vec![
    //         Statement::Assign {
    //             var: "v1".to_owned(),
    //             value: 278,
    //         },
    //         Statement::Assign {
    //             var: "v2".to_owned(),
    //             value: 34,
    //         },
    //         Statement::Operation {
    //             dest: "v3".to_owned(),
    //             op: BinOp::Add,
    //             lhs: "v1".to_owned(),
    //             rhs: "v2".to_owned(),
    //         },
    //     ];
    //     let expected = ControlFlowGraph(HashMap::from([(0, control_block)]));

    //     Ok(())
    // }
}
