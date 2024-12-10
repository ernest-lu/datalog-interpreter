use std::collections::HashSet;
use std::panic;
use std::path::Display;

use bril2json::parse_abstract_program_from_read;
use datalogint::implem::run_datalog;
use datalogint::optimize_bril::{self, perform_liveness_analysis};
use datalogint::parse::{parse_fact_vector, parse_program, Token};
use datalogint::parse_bril::{
    bril_to_string, convert_abstract_program_to_bril_program, get_facts_from_bril_fn, parse_bril,
};
use logos::Logos;
use wasm_bindgen::prelude::*;
extern crate console_error_panic_hook;

#[wasm_bindgen]
#[derive(Clone)]
pub struct StringPair {
    inner: (String, String),
}

#[wasm_bindgen]
impl StringPair {
    #[wasm_bindgen(constructor)]
    pub fn new(first: String, second: String) -> Self {
        StringPair {
            inner: (first, second),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn first(&self) -> String {
        self.inner.0.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_first(&mut self, first: String) {
        self.inner.0 = first;
    }

    #[wasm_bindgen(getter)]
    pub fn second(&self) -> String {
        self.inner.1.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_second(&mut self, second: String) {
        self.inner.1 = second;
    }
}

impl std::fmt::Display for StringPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.inner.0, self.inner.1)
    }
}

#[wasm_bindgen]
pub fn analyze_bril_program(bril_src: &str) -> Result<StringPair, JsError> {
    // Parse the Bril program
    let abstract_prog = parse_abstract_program_from_read(bril_src.as_bytes(), false, false, None);

    let bb_program = convert_abstract_program_to_bril_program(abstract_prog).unwrap();
    let mut running_str = String::new();
    for func in &bb_program.func_index {
        let facts = get_facts_from_bril_fn(func);
        running_str = format!("{}\n{}:", running_str, func.name.clone())
            + &facts
                .iter()
                .fold(String::new(), |acc, fact| format!("{}\n{}", acc, fact));
    }

    let opt_prog = perform_liveness_analysis(bb_program);

    let bril_prog = bril_to_string(&opt_prog);

    Ok(StringPair::new(running_str.clone(), bril_prog))
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
    panic::set_hook(Box::new(console_error_panic_hook::hook));
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
        println!("Result: {}", &result.unwrap());
    }

    #[test]
    fn test_analyze_bril_program_2() {
        let bril_src = r#"
        # Nested loops example
        @main(n: int) {
            zero: int = const 0;
            one: int = const 1;
            i: int = const 0;
        .outer_loop:
            cond1: bool = lt i n;
            br cond1 .outer_body .done;
        .outer_body:
            j: int = const 0;
        .inner_loop:
            cond2: bool = lt j n;
            br cond2 .inner_body .outer_next;
        .inner_body:
            sum: int = add i j;
            j: int = add j one;
            jmp .inner_loop;
        .outer_next:
            i: int = add i one;
            jmp .outer_loop;
        .done:
            ret i;
        }
        "#;
        let result = crate::analyze_bril_program(bril_src);
        assert!(result.is_ok());
        println!("Result: {}", &result.unwrap());
    }
}
