use std::collections::HashSet;

use datalogint::bril_rs::BBProgram;
use datalogint::implem::run_datalog;
use datalogint::optimize_bril::perform_liveness_analysis;
use datalogint::parse::{parse_fact_vector, parse_program, Token};
use datalogint::parse_bril::{bril_to_string, parse_bril};
use logos::Logos;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn analyze_bril_program(bril_src: &str) -> Result<String, JsError> {
    // Parse the Bril program
    // let bril_json = bril2json(bril_src);
    let program = match parse_bril(&bril_src) {
        Ok(p) => p,
        Err(e) => return Err(JsError::new(&format!("Error parsing Bril: {}", e))),
    };
    let prog = perform_liveness_analysis(program);
    Ok(bril_to_string(&prog))
}

#[wasm_bindgen]
pub fn run_datalog_analysis(rules: &str, facts: &str) -> Result<String, JsError> {
    // Parse program (rules)
    let mut rules_lexer = Token::lexer(rules);
    let program = match parse_program(&mut rules_lexer) {
        Ok(program) => program,
        Err(e) => return Err(JsError::new(&format!("Error parsing rules: {}", e))),
    };

    let output_fact_names = program
        .decls
        .iter()
        .filter_map(|decl| {
            if decl.kind == datalogint::parse::DeclKind::Output {
                Some(decl.name.clone())
            } else {
                None
            }
        })
        .collect::<HashSet<String>>();

    // Parse facts
    let mut facts_lexer = Token::lexer(facts);
    let facts = match parse_fact_vector(&mut facts_lexer) {
        Ok(facts) => facts,
        Err(e) => return Err(JsError::new(&format!("Error parsing facts: {}", e))),
    };

    // Run the analysis
    let output_facts = match run_datalog(&program, facts) {
        Ok(output_facts) => output_facts,
        Err(e) => return Err(JsError::new(&format!("Error running analysis: {}", e))),
    };

    // Format the output facts as a string
    let result = output_facts
        .iter()
        .filter(|fact| output_fact_names.contains(&fact.name))
        .fold(String::new(), |acc, fact| format!("{}{}\n", acc, fact));

    Ok(result)
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
