use std::fs::{read_to_string, write};
use std::process::Command;

mod ast;
mod cfg;
mod codegen;
mod parser;
mod symantic_check;
mod symbol_table;
mod tokenizer;

const FILE_ASM: &str = "out.s";
const FILE_EXE: &str = "out";

fn main() {
    let s = read_to_string("test/return.c").unwrap();
    let tokens = tokenizer::tokenize(&s).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    symantic_check::check_syntax(&ast).unwrap();
    let cfg = cfg::ControlFlowGraph::from(&ast);
    let asm = codegen::cfg_to_asm(&cfg).unwrap().join("\n");

    write(FILE_ASM, asm).expect(format!("Failed to write {}", FILE_ASM).as_str());

    let result = Command::new("gcc")
        .args([FILE_ASM, "-o", FILE_EXE])
        .output()
        .expect("Failed to execute `as`");

    println!("result: {:?}", result.status)
}
