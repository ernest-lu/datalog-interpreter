use logos::Logos;
use parse::{parse_program, Token};
// Baseline datalog interpreter for .dl files
// .dl files are defined by the following grammar:
// .decl <var_name>(param 1, param 2, param 3) .input / .output
// each declaration defines a variable, a set of parameters to construct the variable,
// and whether it is an input or output variable

// What follows is a set of rules defined by:
// .rule <head> :- <body>
// where a set of rules infers the head fact from the body facts

mod parse;
fn main() {
    let filename = "samples/sample.dl";
    let src = std::fs::read_to_string(filename).expect("Error reading file");
    let program = parse_program(&mut Token::lexer(&src)).unwrap();
    dbg!(program);
}
