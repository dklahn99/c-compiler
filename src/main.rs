use std::fs::read_to_string;

mod ast;
mod parser;
// mod symantic_check;
mod tokenizer;

fn main() {
    let s = read_to_string("test/main.c").unwrap();
    let tokens = tokenizer::tokenize(&s).unwrap();
    let ast = parser::parse(&tokens);
    println!("{:?}", ast);
}
