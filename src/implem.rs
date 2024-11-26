use crate::parse::{DeclKind, Declaration, Fact, FactLike, Program, Rule};
use std::collections::{BTreeMap, HashMap, HashSet};

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

// a fact is characterized by a mapping from relation name to a set of parameters
// A table is a list of facts with the same relation name
// when we run datalog, we are "joining" the tables in the rules on a specified set of keys
#[derive(Debug)]
struct Table {
    name: String,
    facts: Vec<Fact>,
}

// a mapping of parameter names to parameter names, to be hash joined with a table
#[derive(Clone, Debug)]
pub struct ParameterMapping {
    // the parameter mappings that are being joined on
    // we use a BTreeMap to because it can be hashed
    parameter_maps: HashSet<BTreeMap<String, String>>,
    // the parameter names that are being joined on
    parameter_keys: HashSet<String>,
}

impl ParameterMapping {
    fn is_empty(&self) -> bool {
        self.parameter_keys.is_empty()
    }
    fn new() -> ParameterMapping {
        ParameterMapping {
            parameter_maps: HashSet::new(),
            parameter_keys: HashSet::new(),
        }
    }
}

fn join_parameter_mapping(a: &ParameterMapping, b: &ParameterMapping) -> ParameterMapping {
    if a.is_empty() {
        return b.clone();
    } else if b.is_empty() {
        return a.clone();
    }

    let intsct_keys = a
        .parameter_keys
        .intersection(&b.parameter_keys)
        .collect::<Vec<_>>();

    if intsct_keys.is_empty() {
        return ParameterMapping::new();
    }

    let mut a_hashmap = HashMap::new();
    for pm in a.parameter_maps.iter() {
        let hash = intsct_keys
            .iter()
            .map(|k| pm.get(*k).unwrap().clone())
            .collect::<Vec<_>>();
        a_hashmap.entry(hash).or_insert(vec![]).push(pm);
    }

    let mut b_hashmap = HashMap::new();
    for pm in b.parameter_maps.iter() {
        let hash = intsct_keys
            .iter()
            .map(|k| pm.get(*k).unwrap().clone())
            .collect::<Vec<_>>();
        b_hashmap.entry(hash).or_insert(vec![]).push(pm);
    }

    let mut new_parameter_mapping = ParameterMapping::new();
    for (k, a_v) in a_hashmap {
        if let Some(b_v) = b_hashmap.get(&k) {
            for a_pm in a_v {
                for b_pm in b_v {
                    let mut new_pm = a_pm.clone();
                    new_pm.extend(b_pm.iter().map(|(k, v)| (k.clone(), v.clone())));
                    new_parameter_mapping.parameter_maps.insert(new_pm);
                }
            }
        }
    }

    new_parameter_mapping.parameter_keys = a
        .parameter_keys
        .union(&b.parameter_keys)
        .map(|k| k.clone())
        .collect::<HashSet<_>>();
    new_parameter_mapping
}

struct Database {
    tables: HashMap<String, Table>,
}

fn extend_database(database: &mut Database, facts: &Vec<Fact>) -> () {
    for fact in facts {
        database
            .tables
            .entry(fact.name.clone())
            .or_insert(Table {
                name: fact.name.clone(),
                facts: vec![],
            })
            .facts
            .push(fact.clone());
    }
}

fn get_parameter_mapping(table: &Table, parameter_keys: &Vec<String>) -> ParameterMapping {
    let mut parameter_mapping = ParameterMapping::new();
    for fact in &table.facts {
        parameter_mapping.parameter_maps.insert(
            parameter_keys
                .iter()
                .zip(fact.params().iter())
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        );
    }
    parameter_mapping.parameter_keys = parameter_keys.clone().into_iter().collect();
    parameter_mapping
}

fn get_output_fact(rule: &Rule, parameter_mapping: &ParameterMapping) -> Vec<Fact> {
    let mut facts = HashSet::new();
    for pm in parameter_mapping.parameter_maps.iter() {
        facts.insert(Fact {
            name: rule.head.name.clone(),
            params: rule
                .head
                .params
                .iter()
                .map(|p| pm.get(p).unwrap().clone())
                .collect(),
        });
    }
    facts.into_iter().collect()
}

pub fn run_datalog(program: &Program, input: Vec<Fact>) -> Result<Vec<Fact>, String> {
    verify_facts(program, &input)?;

    // the new frontier consists of facts just recently added
    let mut frontier = Database {
        tables: HashMap::new(),
    };
    extend_database(&mut frontier, &input);
    // semi-naive evaluation runs new X old, new X new, and old X new to join facts together
    let mut facts_hashset: HashSet<Fact> = HashSet::from_iter(input.iter().cloned());

    loop {
        let mut new_facts = vec![];
        for rule in &program.rules {
            let mut good_rule = true;
            let mut current_parameter_mapping = ParameterMapping::new();
            for decl in rule.body.iter() {
                if let Some(table) = frontier.tables.get(&decl.name) {
                    let parameter_mapping = get_parameter_mapping(table, &decl.params);
                    current_parameter_mapping =
                        join_parameter_mapping(&current_parameter_mapping, &parameter_mapping);
                } else {
                    good_rule = false;
                    break;
                }
            }
            if good_rule {
                let output_facts = get_output_fact(rule, &current_parameter_mapping);
                for new_fact in output_facts {
                    if !facts_hashset.contains(&new_fact) {
                        new_facts.push(new_fact);
                    }
                }
            }
        }
        if new_facts.is_empty() {
            break;
        }
        facts_hashset.extend(new_facts.iter().cloned());
        extend_database(&mut frontier, &new_facts);
    }

    // println!("{:?}", facts_hashset);
    Ok(facts_hashset.into_iter().collect())
}
mod tests {
    use super::{join_parameter_mapping, ParameterMapping};
    use std::collections::{BTreeMap, HashSet};

    #[test]
    fn test_join_parameter_mapping() {
        let a = ParameterMapping {
            parameter_maps: HashSet::from([
                BTreeMap::from([
                    ("a_key".to_string(), "a_value".to_string()),
                    ("b_key".to_string(), "b_value".to_string()),
                    ("c_key".to_string(), "c_value".to_string()),
                ]),
                BTreeMap::from([
                    ("a_key".to_string(), "a_value2".to_string()),
                    ("b_key".to_string(), "b_value2".to_string()),
                    ("c_key".to_string(), "c_value2".to_string()),
                ]),
            ]),
            parameter_keys: HashSet::from([
                "a_key".to_string(),
                "b_key".to_string(),
                "c_key".to_string(),
            ]),
        };
        let b = ParameterMapping {
            parameter_maps: HashSet::from([
                BTreeMap::from([
                    ("a_key".to_string(), "a_value".to_string()),
                    ("b_key".to_string(), "b_value".to_string()),
                    ("d_key".to_string(), "d_value".to_string()),
                ]),
                BTreeMap::from([
                    ("a_key".to_string(), "a_value2".to_string()),
                    ("b_key".to_string(), "b_value2".to_string()),
                    ("d_key".to_string(), "d_value2".to_string()),
                ]),
            ]),
            parameter_keys: HashSet::from([
                "a_key".to_string(),
                "b_key".to_string(),
                "d_key".to_string(),
            ]),
        };
        let c = join_parameter_mapping(&a, &b);

        assert_eq!(c.parameter_maps.len(), 2);
        assert_eq!(c.parameter_keys.len(), 4);
        assert!(c.parameter_maps.contains(&BTreeMap::from([
            ("a_key".to_string(), "a_value".to_string()),
            ("b_key".to_string(), "b_value".to_string()),
            ("c_key".to_string(), "c_value".to_string()),
            ("d_key".to_string(), "d_value".to_string()),
        ])));

        assert!(c.parameter_maps.contains(&BTreeMap::from([
            ("a_key".to_string(), "a_value2".to_string()),
            ("b_key".to_string(), "b_value2".to_string()),
            ("c_key".to_string(), "c_value2".to_string()),
            ("d_key".to_string(), "d_value2".to_string()),
        ])));
    }
}
