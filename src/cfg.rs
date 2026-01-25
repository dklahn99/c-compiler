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

type VarName = u64;
type Label = u64;

pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

pub enum Statement {
    If {
        var: VarName,
        goto_true: Label,
        goto_false: Label,
    },
    Goto(Label),
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

pub struct ControlBlock {
    label: Label,
    statements: Vec<Statement>,
}

mod tests {
    use super::*;

    #[test]
    fn my_test() {}
}
