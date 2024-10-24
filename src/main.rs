mod implem;
mod parse;
use implem::run_datalog;
use logos::Logos;
use parse::{parse_fact_vector, parse_program, Token};
// Baseline datalog interpreter for .dl files
// .dl files are defined by the following grammar:
// .decl <var_name>(param 1, param 2, param 3) .input / .output
// each declaration defines a variable, a set of parameters to construct the variable,
// and whether it is an input or output variable

// What follows is a set of rules defined by:
// .rule <head> :- #num_rules <body>
// where a set of rules infers the head fact from the body facts

// The list of facts is the input to the program
// The facts are given in a .in file with the following format:
// <num_facts>
// <fact_1>
// <fact_2>
// ...

fn main() {
    let filename = "samples/sample.dl";
    let src = std::fs::read_to_string(filename).expect("Error reading file");
    let program = parse_program(&mut Token::lexer(&src)).unwrap();

    let input_filename = "samples/sample.in";
    let src_input = std::fs::read_to_string(input_filename).expect("Error reading file");
    let facts = parse_fact_vector(&mut Token::lexer(&src_input)).unwrap();

    dbg!(&facts);

    let result = run_datalog(program, facts).unwrap();
    dbg!(result);
}
