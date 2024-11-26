use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::iter::successors;
use std::{convert, vec};

use crate::bril_rs::{
    load_program_from_read, BBFunction, BBProgram, BasicBlock, Code, EffectOps, Function,
    Instruction, Program,
};
use crate::parse::Fact;

// copyiing over bril_rs because we can't use it as a cargo dependency // has platform specific code
// pub mod bril_rs {
//     struct Program {}
// }

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

fn get_block_name(block: &BasicBlock) -> String {
    block.label.clone().unwrap_or("default_block".to_string())
}

fn get_instr_name(block_name: &str, i: usize) -> String {
    block_name.to_string() + "_instr_" + &i.to_string()
}

fn get_instr_successors(bril_fn: &BBFunction) -> HashMap<String, Vec<String>> {
    let mut instr_successors = HashMap::new();
    let num_basic_blocks = bril_fn.blocks.len();

    for (basic_block_idx, block) in bril_fn.blocks.iter().enumerate() {
        let block_name = get_block_name(block);
        for (i, instr) in block.instrs.iter().enumerate() {
            let instr_name = get_instr_name(&block_name, i);

            // next instr
            if i < block.instrs.len() - 1 {
                let next_instr_name = get_instr_name(&block_name, i + 1);
                instr_successors
                    .entry(instr_name.clone())
                    .or_insert(Vec::new())
                    .push(next_instr_name);
            } else {
                // last instr point to the next block
                if basic_block_idx < num_basic_blocks - 1 {
                    let next_block_name = get_block_name(&bril_fn.blocks[basic_block_idx + 1]);
                    instr_successors
                        .entry(instr_name.clone())
                        .or_insert(Vec::new())
                        .push(get_instr_name(&next_block_name, 0));
                }
            }

            // successor instructions
            match instr {
                Instruction::Effect {
                    op: EffectOps::Jump | EffectOps::Branch,
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

fn get_instr_uses(instr: &Instruction) -> Vec<String> {
    match instr {
        Instruction::Value { args, .. } | Instruction::Effect { args, .. } => args.clone(),
        _ => vec![], // empty vector for other instructions
    }
}

fn get_all_defined_vars(bril_fn: &BBFunction) -> HashSet<String> {
    let mut defined_vars = HashSet::new();
    for block in &bril_fn.blocks {
        for (i, instr) in block.instrs.iter().enumerate() {
            match instr {
                Instruction::Constant { dest, .. } | Instruction::Value { dest, .. } => {
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
                Instruction::Constant { dest, .. } | Instruction::Value { dest, .. } => {
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

fn convert_bb_fn_to_bril_fn(bb_fn: &BBFunction) -> Function {
    let instrs: Vec<Code> = bb_fn
        .blocks
        .iter()
        .flat_map(|block| {
            let mut instructions = Vec::new();
            if let Some(label) = &block.label {
                instructions.push(Code::Label {
                    label: label.clone(),
                    pos: None,
                });
            }
            instructions.extend(block.instrs.iter().cloned().map(Code::Instruction));
            instructions
        })
        .collect();
    Function {
        args: bb_fn.args.clone(),
        return_type: bb_fn.return_type.clone(),
        name: bb_fn.name.clone(),
        pos: bb_fn.pos.clone(),
        instrs: instrs,
    }
}

pub fn bril_to_string(program: &BBProgram) -> String {
    let convert_prog_back = Program {
        functions: program
            .func_index
            .iter()
            .map(convert_bb_fn_to_bril_fn)
            .collect(),
    };

    serde_json::to_string_pretty(&convert_prog_back).unwrap()
}
