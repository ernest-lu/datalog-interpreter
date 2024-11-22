use datalogint::parse::Fact;
use datalogint::*;
use logos::Logos;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run_datalog_analysis(input: &str) -> String {
    // Parse input using the existing parser
    let mut lexer = Token::lexer(input);
    match parse::parse_fact_vector(&mut lexer) {
        Ok(facts) => {
            // Create a simple program that defines edge and path relations
            let program = Program {
                decls: vec![
                    parse::Declaration {
                        name: "edge".to_string(),
                        params: vec!["x".to_string(), "y".to_string()],
                        kind: parse::DeclKind::Input,
                    },
                    parse::Declaration {
                        name: "path".to_string(),
                        params: vec!["x".to_string(), "y".to_string()],
                        kind: parse::DeclKind::Output,
                    },
                ],
                rules: vec![
                    // Base case: edge(x,y) -> path(x,y)
                    parse::Rule {
                        head: parse::Declaration {
                            name: "path".to_string(),
                            params: vec!["x".to_string(), "y".to_string()],
                            kind: parse::DeclKind::Output,
                        },
                        body: vec![parse::Declaration {
                            name: "edge".to_string(),
                            params: vec!["x".to_string(), "y".to_string()],
                            kind: parse::DeclKind::Input,
                        }],
                    },
                    // Transitive case: path(x,y) ∧ edge(y,z) -> path(x,z)
                    parse::Rule {
                        head: parse::Declaration {
                            name: "path".to_string(),
                            params: vec!["x".to_string(), "z".to_string()],
                            kind: parse::DeclKind::Output,
                        },
                        body: vec![
                            parse::Declaration {
                                name: "path".to_string(),
                                params: vec!["x".to_string(), "y".to_string()],
                                kind: parse::DeclKind::Output,
                            },
                            parse::Declaration {
                                name: "edge".to_string(),
                                params: vec!["y".to_string(), "z".to_string()],
                                kind: parse::DeclKind::Input,
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
