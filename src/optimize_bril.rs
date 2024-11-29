use crate::bril_rs::load_program_from_read;
use crate::bril_rs::Instruction;
use crate::bril_rs::{BBFunction, BBProgram, BasicBlock};
use crate::implem::run_datalog;
use crate::parse::{parse_program, Fact, Program, Token};
use crate::parse_bril::{bril_to_string, get_facts_from_bril_fn, parse_bril};
use logos::Logos;
use std::collections::{HashMap, HashSet};

const LIVENESS_RULES_FILENAME: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/samples/dataflow/liveness/liveness.dl"
);

pub fn perform_liveness_analysis(mut bril_program: BBProgram) -> BBProgram {
    let datalog_rules_src =
        std::fs::read_to_string(LIVENESS_RULES_FILENAME).expect("Error reading file");
    let datalog_program = parse_program(&mut Token::lexer(&datalog_rules_src)).unwrap();
    for func in &mut bril_program.func_index {
        let facts = get_facts_from_bril_fn(func);
        let output_facts = run_datalog(&datalog_program, facts).unwrap();

        let facts_out = output_facts
            .iter()
            .filter(|f| f.name == "var_live")
            .collect::<Vec<_>>();

        let mut live_by_line: HashMap<String, HashSet<String>> = HashMap::new();

        // if definition and variable is not live after the defintion, the definition can be removed

        for fact in facts_out {
            live_by_line
                .entry(fact.params[0].clone())
                .or_insert(HashSet::new())
                .insert(fact.params[1].clone());
        }

        for block in &mut func.blocks {
            let block_name = block.label.clone().unwrap_or("default_block".to_string());
            let mut new_instrs = vec![];
            for (i, instr) in block.instrs.iter().enumerate() {
                let instr_name = block_name.clone() + "_instr_" + &i.to_string();
                let mut can_remove_instr = false;
                match instr {
                    Instruction::Value { dest, .. } | Instruction::Constant { dest, .. } => {
                        can_remove_instr = live_by_line
                            .get(&instr_name)
                            .map_or(true, |vars| !vars.contains(dest));
                    }
                    _ => {}
                }
                if !can_remove_instr {
                    new_instrs.push(instr.clone());
                }
            }
            block.instrs = new_instrs;
        }
    }
    bril_program
}
