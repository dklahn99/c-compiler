mod tokenizer;

fn main() {
    let s = "(  \n)";
    let no_whitespace: String = s.chars().filter(|c| !c.is_whitespace()).collect();

    println!("{:?}", tokenizer::tokenize(&no_whitespace));
}
