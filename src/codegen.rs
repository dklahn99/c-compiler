use crate::cfg::*;
use std::collections::HashMap;
use std::fmt;

/*
    For now, we'll just assign variables to a few registerss:
    v1: rbx
    v2: rcx
    v3: rdx
    v4-v11: r8-r15
*/
const ASM_HEADER: [&'static str; 2] = [".global main", "main:"];

enum RegisterGP {
    RBX,
    RCX,
    RDX,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl fmt::Display for RegisterGP {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            RegisterGP::RBX => "rbx",
            RegisterGP::RCX => "rcx",
            RegisterGP::RDX => "rdx",
            RegisterGP::R9 => "r9",
            RegisterGP::R10 => "r10",
            RegisterGP::R11 => "r11",
            RegisterGP::R12 => "r12",
            RegisterGP::R13 => "r13",
            RegisterGP::R14 => "r14",
            RegisterGP::R15 => "r15",
            _ => "",
        };
        write!(f, "{}", s)
    }
}

fn var_to_reg(var: &CfgVarName) -> Result<RegisterGP, String> {
    match var.as_str() {
        "v1" => Ok(RegisterGP::RBX),
        "v2" => Ok(RegisterGP::RCX),
        "v3" => Ok(RegisterGP::RDX),
        "v4" => Ok(RegisterGP::R8),
        "v5" => Ok(RegisterGP::R9),
        _ => Err(format!("Could not map var {}", var)),
    }
}

fn assign_to_asm(var: &CfgVarName, value: u64) -> Result<Vec<String>, String> {
    Ok(vec![format!("mov ${}, %{}", value, var_to_reg(var)?)])
}

fn return_to_asm(var: &CfgVarName) -> Result<Vec<String>, String> {
    Ok(vec![format!("mov %{}, %rax", var_to_reg(var)?)])
}

pub fn cfg_to_asm(cfg: &crate::cfg::ControlFlowGraph) -> Result<Vec<String>, String> {
    assert_eq!(cfg.len(), 1); // Right now we're only considering programs with no control flow. These programs should have one control block
    assert!(cfg.contains_key(&0)); // The one control block should have ID 0

    let block = cfg.get(&0).unwrap();
    let mut asm: Vec<String> = ASM_HEADER.iter().map(|&s| s.to_owned()).collect();
    for s in block {
        let statement_asm = match s {
            Statement::Assign { var, value } => assign_to_asm(var, *value)?,
            Statement::Return(var) => return_to_asm(var)?,
            _ => return Err("".to_owned()),
        };
        asm.extend(statement_asm);
    }
    Ok(asm)
}

mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::symantic_check::check_syntax;
    use crate::tokenizer::tokenize;
    use std::fs::read_to_string;

    #[test]
    fn codegen_integration_return() -> Result<(), String> {
        let s = read_to_string("test/return.c").unwrap();
        let tokens = tokenize(&s)?;
        let ast = parse(&tokens)?;
        check_syntax(&ast)?;
        let cfg = ControlFlowGraph::from(&ast);
        let asm = cfg_to_asm(&cfg)?;

        println!("CFG: {:?}", cfg);
        let expected = vec![".global main", "main:", "mov $123, %rbx", "mov %rbx, %rax"];
        assert_eq!(asm, expected);

        Ok(())
    }
}
