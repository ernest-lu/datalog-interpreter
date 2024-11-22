use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::iter::successors;
use std::vec;

use crate::parse::Fact;
use bril_rs::{load_program_from_read, output_program};
use brilirs::basic_block::BBFunction;
use brilirs::basic_block::BBProgram;

pub fn parse_bril(input: &str) -> Result<BBProgram, String> {
    let program = load_program_from_read(input.as_bytes());
    match program.try_into() {
        Ok(p) => Ok(p),
        Err(e) => Err(format!(
            "Error converting bril program to basic block program: {}",
            e
        )),
    }
}

fn get_instr_successors(bril_fn: &BBFunction) -> HashMap<String, Vec<String>> {
    let mut instr_successors = HashMap::new();
    for block in &bril_fn.blocks {
        let block_name = block.label.clone().unwrap_or("default_block".to_string());
        for (i, instr) in block.instrs.iter().enumerate() {
            let instr_name = block_name.clone() + "_instr_" + &i.to_string();

            // next instr
            if i < block.instrs.len() - 1 {
                let next_instr_name = block_name.clone() + "_instr_" + &(i + 1).to_string();
                instr_successors
                    .entry(instr_name.clone())
                    .or_insert(Vec::new())
                    .push(next_instr_name);
            }

            // successor instructions
            match instr {
                bril_rs::Instruction::Effect {
                    op: bril_rs::EffectOps::Jump | bril_rs::EffectOps::Branch,
                    labels,
                    ..
                } => {
                    for label in labels {
                        instr_successors
                            .entry(instr_name.clone())
                            .or_insert(Vec::new())
                            .push(label.clone() + "_instr_0");
                    }
                }
                _ => {}
            }
        }
    }
    instr_successors
}

fn get_instr_uses(instr: &bril_rs::Instruction) -> Vec<String> {
    match instr {
        bril_rs::Instruction::Value { args, .. } | bril_rs::Instruction::Effect { args, .. } => {
            args.clone()
        }
        _ => vec![], // empty vector for other instructions
    }
}

fn get_all_defined_vars(bril_fn: &BBFunction) -> HashSet<String> {
    let mut defined_vars = HashSet::new();
    for block in &bril_fn.blocks {
        for (i, instr) in block.instrs.iter().enumerate() {
            match instr {
                bril_rs::Instruction::Constant { dest, .. }
                | bril_rs::Instruction::Value { dest, .. } => {
                    defined_vars.insert(dest.clone());
                }
                _ => {}
            }
        }
    }
    defined_vars
}

// implement successor, undefined, var_used
pub fn get_facts_from_bril_fn(bril_fn: &BBFunction) -> Vec<Fact> {
    let instr_successors = get_instr_successors(bril_fn);
    let var_names = get_all_defined_vars(bril_fn);

    // add var_used
    let mut output_facts = Vec::new();
    for block in &bril_fn.blocks {
        let block_name = block.label.clone().unwrap_or("default_block".to_string());
        for (i, instr) in block.instrs.iter().enumerate() {
            let instr_name = block_name.clone() + "_instr_" + &i.to_string();
            for arg in get_instr_uses(instr) {
                if var_names.contains(&arg) {
                    output_facts.push(Fact {
                        name: "var_used".to_string(),
                        params: vec![instr_name.clone(), arg.clone()],
                    });
                }
            }
        }
    }

    // add undefined
    for block in &bril_fn.blocks {
        let block_name = block.label.clone().unwrap_or("default_block".to_string());
        for (i, instr) in block.instrs.iter().enumerate() {
            let instr_name = block_name.clone() + "_instr_" + &i.to_string();
            let instr_defineed_vars = match instr {
                bril_rs::Instruction::Constant { dest, .. }
                | bril_rs::Instruction::Value { dest, .. } => {
                    HashSet::from_iter(vec![dest.clone()])
                }
                _ => HashSet::new(),
            };
            for var in var_names.iter() {
                if !instr_defineed_vars.contains(var) {
                    output_facts.push(Fact {
                        name: "undefined".to_string(),
                        params: vec![instr_name.clone(), var.clone()],
                    });
                }
            }
        }
    }

    // add successor
    for (instr_name, successors) in instr_successors.iter() {
        for successor in successors {
            output_facts.push(Fact {
                name: "successor".to_string(),
                params: vec![instr_name.clone(), successor.clone()],
            });
        }
    }
    output_facts
}

fn output_bril_program(program: &BBProgram) -> () {}
