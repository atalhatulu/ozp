use std::fs;

fn main() {
    let kod = fs::read_to_string("examples/fibonacci_tipli.ozp").unwrap();
    let tokens = ozc::lexer::tokenize(&kod);
    let ast = ozc::parser::parse(&tokens).unwrap();
    println!("{:#?}", ast);
}
