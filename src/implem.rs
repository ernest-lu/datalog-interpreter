use crate::parse::{DeclKind, Fact, FactLike, Program};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};

//  Verify that the facts are valid according to the program
//  Facts have correct number of arguments
//  Facts are made up of existing declarations
fn verify_facts(program: &Program, facts: &Vec<Fact>) -> Result<(), String> {
    let mut decl_map = HashMap::new();

    for decl in &program.decls {
        decl_map.insert(decl.name.clone(), (decl.params.len(), decl.kind));
    }

    for fact in facts {
        let Some((num_params, kind)) = decl_map.get(&fact.name) else {
            return Err(format!("Fact {} not declared", fact.name));
        };

        if *kind == DeclKind::Output {
            return Err(format!("Fact {} is not an input fact", fact.name));
        }

        if fact.params.len() != *num_params {
            return Err(format!(
                "Fact {} has the wrong number of parameters",
                fact.name
            ));
        }
    }
    Ok(())
}

fn get_parameter_locations<T>(facts: &Vec<T>) -> HashMap<String, HashSet<(usize, usize)>>
where
    T: FactLike,
{
    let mut parameter_locations = HashMap::new();
    for (i, fact) in facts.iter().enumerate() {
        for (j, param) in fact.params().iter().enumerate() {
            parameter_locations
                .entry(param.clone())
                .or_insert(HashSet::new())
                .insert((i, j));
        }
    }
    parameter_locations
}

pub fn run_datalog(program: Program, input: Vec<Fact>) -> Result<Vec<Fact>, String> {
    verify_facts(&program, &input)?;

    let mut facts_hashset: HashSet<Fact> = HashSet::from_iter(input.iter().cloned());

    // (R X P)
    // ((r + R) X (p + P))
    // rp, Rp, RP, rP

    let mut previous_frontier: HashSet<Fact> = facts_hashset.clone();
    let mut old_frontier: HashSet<Fact> = HashSet::new();

    loop {
        // loop until convergence
        let mut new_facts = vec![];
        for rule in &program.rules {
            let num_body_facts = rule.body.len();
            let expected_parameter_locations = get_parameter_locations(&rule.body);

            for size_new_frontier in 1..=num_body_facts {
                // iterate at least one new fact from the previous frontier for semi-naive evaluation
                let size_old_frontier = num_body_facts - size_new_frontier;
                for subset_new in previous_frontier.iter().combinations(size_new_frontier) {
                    for subset_old in old_frontier.iter().combinations(size_old_frontier) {
                        let combined_subset: Vec<Fact> = [
                            subset_new.iter().map(|f| (*f).clone()).collect::<Vec<_>>(),
                            subset_old.iter().map(|f| (*f).clone()).collect::<Vec<_>>(),
                        ]
                        .concat();
                        for subset in combined_subset.into_iter().permutations(num_body_facts) {
                            // Check if each fact name matches the corresponding body declaration
                            let matches = subset.iter().zip(&rule.body).all(|(fact, decl)| {
                                fact.name == decl.name && fact.params.len() == decl.params.len()
                            });

                            if matches {
                                let parameter_locations = get_parameter_locations(&subset);

                                let mut parameter_name_mappings = HashMap::new();
                                let mut okay = true;
                                for (i, decl) in rule.body.iter().enumerate() {
                                    for (j, param) in decl.params.iter().enumerate() {
                                        if parameter_name_mappings.contains_key(param) {
                                            continue;
                                        }
                                        let actual_parameter_name = &subset[i].params()[j];
                                        let expected_parameter_hashset =
                                            expected_parameter_locations.get(param).unwrap();
                                        let actual_parameter_hashset =
                                            parameter_locations.get(actual_parameter_name).unwrap();
                                        if expected_parameter_hashset != actual_parameter_hashset {
                                            okay = false;
                                            break;
                                        }
                                        parameter_name_mappings
                                            .insert(param.clone(), actual_parameter_name.clone());
                                    }
                                    if !okay {
                                        break;
                                    }
                                }
                                if okay {
                                    let new_fact = Fact {
                                        name: rule.head.name.clone(),
                                        params: rule
                                            .head
                                            .params
                                            .iter()
                                            .map(|p| {
                                                parameter_name_mappings.get(p).unwrap().clone()
                                            })
                                            .collect(),
                                    };
                                    if !facts_hashset.contains(&new_fact) {
                                        new_facts.push(new_fact);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if new_facts.is_empty() {
            break;
        }
        // extend the old frontier with the previous frontier
        old_frontier.extend(previous_frontier.clone());
        previous_frontier = HashSet::from_iter(new_facts.iter().cloned());
        facts_hashset.extend(new_facts.iter().cloned());
    }

    println!("{:?}", facts_hashset);
    Ok(facts_hashset.into_iter().collect())
}
