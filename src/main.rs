use std::fs::read_to_string;

mod parser;
mod tokenizer;

fn main() {
    let s = read_to_string("test/if_else.c").unwrap();
    let tokens = tokenizer::tokenize(&s).unwrap();
    let ast = parser::parse(tokens);
    println!("{:?}", ast);
}
