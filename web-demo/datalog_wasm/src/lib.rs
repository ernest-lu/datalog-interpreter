use datalogint::bril_rs::{load_program_from_read, BBProgram};
use datalogint::implem::run_datalog;
use datalogint::optimize_bril::perform_liveness_analysis;
use datalogint::parse::{parse_fact_vector, DeclKind, Declaration, Program, Rule, Token};
use datalogint::parse_bril::{bril_to_string, parse_bril};
use logos::Logos;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::process::{Command, Stdio};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn analyze_bril_program(bril_src: &str) -> Result<String, JsError> {
    // Parse the Bril program
    // let bril_json = bril2json(bril_src);
    let program = match parse_bril(&bril_src) {
        Ok(p) => p,
        Err(e) => return Err(JsError::new(&format!("Error parsing Bril: {}", e))),
    };
    dbg!(&program);
    let prog = perform_liveness_analysis(program);
    Ok(bril_to_string(&prog))
}

#[wasm_bindgen]
pub fn run_datalog_analysis(input: &str) -> String {
    // Parse input using the existing parser
    let mut lexer = Token::lexer(input);
    match parse_fact_vector(&mut lexer) {
        Ok(facts) => {
            // Create a simple program that defines edge and path relations
            let program = Program {
                decls: vec![
                    Declaration {
                        name: "edge".to_string(),
                        params: vec!["x".to_string(), "y".to_string()],
                        kind: DeclKind::Input,
                    },
                    Declaration {
                        name: "path".to_string(),
                        params: vec!["x".to_string(), "y".to_string()],
                        kind: DeclKind::Output,
                    },
                ],
                rules: vec![
                    // Base case: edge(x,y) -> path(x,y)
                    Rule {
                        head: Declaration {
                            name: "path".to_string(),
                            params: vec!["x".to_string(), "y".to_string()],
                            kind: DeclKind::Output,
                        },
                        body: vec![Declaration {
                            name: "edge".to_string(),
                            params: vec!["x".to_string(), "y".to_string()],
                            kind: DeclKind::Input,
                        }],
                    },
                    // Transitive case: path(x,y) ∧ edge(y,z) -> path(x,z)
                    Rule {
                        head: Declaration {
                            name: "path".to_string(),
                            params: vec!["x".to_string(), "z".to_string()],
                            kind: DeclKind::Output,
                        },
                        body: vec![
                            Declaration {
                                name: "path".to_string(),
                                params: vec!["x".to_string(), "y".to_string()],
                                kind: DeclKind::Output,
                            },
                            Declaration {
                                name: "edge".to_string(),
                                params: vec!["y".to_string(), "z".to_string()],
                                kind: DeclKind::Input,
                            },
                        ],
                    },
                ],
            };

            // Run the analysis
            match run_datalog(&program, facts) {
                Ok(result) => {
                    let output = result
                        .iter()
                        .filter(|f| f.name == "path")
                        .fold("".to_string(), |acc, f| {
                            acc.to_owned() + &format!("{}\n", f)
                        });
                    format!("{}", output)
                }
                Err(e) => format!("Analysis error: {}", e),
            }
        }
        Err(e) => format!("Parse error: {}", e),
    }
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_analyze_bril_program() {
        let bril_src = r#"
            @main() {
              x: int = const 3;
              y: int = const 5;
              y: int = add x y;
              print y;
              x: int = const 4;
            }
        "#;
        let result = crate::analyze_bril_program(bril_src);
        assert!(result.is_ok());
    }
}
