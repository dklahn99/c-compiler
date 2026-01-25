use crate::ast::*;
use crate::symantic_check::*;
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

pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}
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

type ControlBlock = Vec<Statement>;
struct ControlFlowGraph(HashMap<ControlBlockId, ControlBlock>);

impl ControlFlowGraph {
    pub fn from(declarations: &Vec<Declaration>, symbol_table: &SymbolTable) -> Self {
        // For now, we're only considering programs with a single declaration: a main function
        assert_eq!(declarations.len(), 1);

        let Declaration::Function {
            name,
            args,
            return_type,
            scope,
        } = &declarations[0];
        assert_eq!(name, "main");

        let mut namer = VarNamer::new();

        // execution should start with ControlBlock zero

        // build CFG. This should be a single block since we don't have any conditionals
        ControlFlowGraph(HashMap::new())
    }
}

struct VarNamer {
    counter: u64,
}

impl VarNamer {
    fn new() -> Self {
        return VarNamer {
            counter: 0,
        }
    }

    fn next(&mut self) -> String {
        self.counter += 1;
        return format!("v{:}", self.counter);
    }
}

mod tests {
    use super::*;
    use std::fs::read_to_string;
    use crate::tokenizer::tokenize;
    use crate::parser::parse;
    use crate::symantic_check::check_syntax;

    #[test]
    fn my_test() -> Result<(), String> {
        let s = read_to_string("test/main.c").unwrap();
        let tokens = tokenize(&s)?;
        let ast = parse(&tokens)?;
        let symbol_table = check_syntax(&ast)?;
        let cfg = ControlFlowGraph::from(&ast, &symbol_table);

        let control_block = vec![
            Statement::Assign{
                var: "v1".to_owned(),
                value: 278,
            },
            Statement::Assign{
                var: "v2".to_owned(),
                value: 34,
            },
            Statement::Operation {
                dest: "v3".to_owned(),
                op: BinOp::Add,
                lhs: "v1".to_owned(),
                rhs: "v2".to_owned(),
            }
        ];
        let expected = ControlFlowGraph (
            HashMap::from([
                (0, control_block),
            ])
        );

        Ok(())
    }
}

