use ozc::lexer::tokenize;
use ozc::parser::parse;
use std::fs;

fn main() {
    let kod = fs::read_to_string("examples/fibonacci_tipli.ozp").unwrap();
    let tokens = tokenize(&kod);
    println!("Tokens: {:?}", tokens.len());
    let ast = parse(&tokens).unwrap();
    println!("Program: {:#?}", ast.komutlar[0]);
}
