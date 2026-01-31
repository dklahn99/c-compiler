use crate::ast;
use crate::symbol_table::VarName;
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
type CfgVarName = String;
type ControlBlockId = u64;

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum Statement {
    // TODO: add in conditional support later
    // If {
    //     var: CfgVarName,
    //     goto_true: ControlBlockId,
    //     goto_false: ControlBlockId,
    // },
    Goto(ControlBlockId),
    Assign {
        var: CfgVarName,
        value: u64,
    },
    Operation {
        dest: CfgVarName,
        op: BinOp,
        lhs: CfgVarName,
        rhs: CfgVarName,
    },
    Return(CfgVarName),
}

/*
 * Eventually, want to be able to map variable name in a scope to a cfg var name
 */
struct CFGBuildContext {
    var_counter: u64,
    var_map: HashMap<VarName, CfgVarName>, // maps Symbol Table var names to CFG var names (e.g. "x" -> "v1")
}

#[allow(dead_code)]
impl CFGBuildContext {
    fn new() -> Self {
        CFGBuildContext {
            var_counter: 0,
            var_map: HashMap::new(),
        }
    }

    fn inc(&mut self) -> CfgVarName {
        self.var_counter += 1;
        format!("v{:}", self.var_counter)
    }

    fn register_var(&mut self, var: CfgVarName) {
        let a = self.inc();
        self.var_map.insert(var, a);
    }

    fn lookup(&self, var: &VarName) -> Option<&CfgVarName> {
        self.var_map.get(var)
    }
}

type ControlBlock = Vec<Statement>;

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
struct ControlFlowGraph(HashMap<ControlBlockId, ControlBlock>);

#[allow(dead_code)]
impl ControlFlowGraph {
    fn new() {}

    pub fn from(declarations: &Vec<ast::Declaration>) -> Self {
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

        // Right now this is just a single block since there are no conditionals
        ControlFlowGraph(HashMap::from([(0, block)]))
    }

    fn process(
        stmt: &ast::Statement,
        context: &mut CFGBuildContext,
    ) -> Result<Vec<Statement>, String> {
        match stmt {
            ast::Statement::VarDeclare { .. } => {
                ControlFlowGraph::process_var_declare(&stmt, context)
            }
            ast::Statement::Return(..) => ControlFlowGraph::process_return(&stmt, context),
            _ => Err("Not Implemented".to_owned()),
        }
    }

    fn process_var_declare(
        stmt: &ast::Statement,
        context: &mut CFGBuildContext,
    ) -> Result<Vec<Statement>, String> {
        if let ast::Statement::VarDeclare {
            name,
            var_type,
            value,
        } = stmt
        {
            assert_eq!(var_type, &ast::Type::Int);

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

    fn process_return(
        stmt: &ast::Statement,
        context: &mut CFGBuildContext,
    ) -> Result<Vec<Statement>, String> {
        if let ast::Statement::Return(expr) = stmt {
            match expr {
                ast::Expr::IntLiteral(i) => {
                    let cfg_var_name = context.inc();
                    return Ok(vec![
                        Statement::Assign {
                            var: cfg_var_name.clone(),
                            value: *i,
                        },
                        Statement::Return(cfg_var_name.clone()),
                    ]);
                }
                ast::Expr::Variable(var_name) => {
                    let cfg_var_name = context.lookup(var_name).expect("");
                    return Ok(vec![Statement::Return(cfg_var_name.clone())]);
                }
                _ => return Err(format!("")),
            };
        };

        Err(format!(""))
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

        let mut context = CFGBuildContext::new();
        assert_eq!(
            ControlFlowGraph::process(&vd, &mut context)?,
            vec![Statement::Assign {
                var: "v1".to_owned(),
                value: 123,
            }]
        );
        assert_eq!(
            ControlFlowGraph::process(&vd, &mut context)?,
            vec![Statement::Assign {
                var: "v2".to_owned(),
                value: 123,
            }]
        );

        Ok(())
    }

    #[test]
    fn test_return_int_literal() -> Result<(), String> {
        let ret = ast::Statement::Return(ast::Expr::IntLiteral(123));
        let mut context = CFGBuildContext::new();
        assert_eq!(
            ControlFlowGraph::process(&ret, &mut context)?,
            vec![
                Statement::Assign {
                    var: "v1".to_owned(),
                    value: 123,
                },
                Statement::Return("v1".to_owned()),
            ]
        );

        Ok(())
    }

    #[test]
    fn test_return_var() -> Result<(), String> {
        let ret = ast::Statement::Return(ast::Expr::Variable("x".to_owned()));

        let mut context = CFGBuildContext::new();
        context.register_var("x".to_owned());

        assert_eq!(
            ControlFlowGraph::process(&ret, &mut context)?,
            vec![Statement::Return("v1".to_owned()),]
        );

        Ok(())
    }

    #[test]
    fn test_cfg_integration() -> Result<(), String> {
        let s = read_to_string("test/return.c").unwrap();
        let tokens = tokenize(&s)?;
        let ast = parse(&tokens)?;
        check_syntax(&ast)?;
        let cfg = ControlFlowGraph::from(&ast);

        println!("CFG: {:?}", cfg);

        let control_block = vec![
            Statement::Assign {
                var: "v1".to_owned(),
                value: 123,
            },
            Statement::Return("v1".to_owned()),
        ];
        let expected = ControlFlowGraph(HashMap::from([(0, control_block)]));

        assert_eq!(cfg, expected);

        Ok(())
    }
}
