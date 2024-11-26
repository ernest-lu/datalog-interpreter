mod bril_rs;
mod implem;
mod optimize_bril;
mod parse;
mod parse_bril;
use implem::run_datalog;
use logos::Logos;
use optimize_bril::perform_liveness_analysis;
use parse::{parse_fact_vector, parse_program, DeclKind, Token};
use parse_bril::{bril_to_string, get_facts_from_bril_fn, parse_bril};
use std::collections::{HashMap, HashSet};
use std::env;
use std::hash::Hash;

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
    let filname = env::args().nth(1).expect("No filename provided");
    // println!("{}", filname);
    let src = std::fs::read_to_string(filname).expect("Error reading file");

    let mut bril_program = parse_bril(&src).unwrap();
    let prog = perform_liveness_analysis(bril_program);

    println!("{}", bril_to_string(&prog));
}
