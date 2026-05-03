use serde_json;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::hash::Hash;

use enum_stringify::EnumStringify;
use log::debug;

mod tree;
use tree::extraction::Process;
mod parser;
use parser::ParserNode;
mod utils;
use utils::output_cleaner;

use crate::tree::extraction::{insert_branches, visit_in_order, SEQUENCE_DIAGRAM_MODE};

#[derive(EnumStringify, Hash, PartialEq, Eq, Debug, Copy, Clone)]
pub enum ProtocolType {
    Real,
    Ideal,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let queries = fs::read_to_string("queries.json")?;
    let queries_list: HashMap<String, HashMap<String, String>> = serde_json::from_str(&queries)?;
    let mut queries_map: HashMap<ProtocolType, HashMap<String, String>> = HashMap::new();
    let variables = fs::read_to_string("variables_mapping.json")?;
    let mut variables_map: HashMap<String, String> = serde_json::from_str(&variables)?;
    let json_data = fs::read_to_string("result_no_sim.json")?;
    let data = serde_json::Deserializer::from_str(&json_data);
    let data: HashMap<String, Vec<ParserNode>> = parser::data_parser(data)?;
    let mut processes: HashMap<String, Process> = HashMap::new();
    for (query_type, mut values) in queries_list.into_iter() {
        match query_type.as_str() {
            "realw" => {
                queries_map.insert(ProtocolType::Real, values);
            }
            "idealw" => {
                // add mapping a->sim to the ideal world queries since it may be missing
                values.insert("a".to_string(), "sim".to_string());
                queries_map.insert(ProtocolType::Ideal, values);
            }
            _ => (),
        }
    }
    for (process_name, messages) in data {
        debug!("Processing process: {}", &process_name);
        let mut new_process = Process::new(process_name.clone(), None);
        new_process.add_messages(&messages);
        processes.insert(process_name, new_process);
    }
    // hashmap to keep track of the variables that are used in the branching nodes for each protocol type
    let mut branches_variables: HashMap<ProtocolType, HashMap<String, String>> = HashMap::new();
    branches_variables.insert(ProtocolType::Real, HashMap::new());
    branches_variables.insert(ProtocolType::Ideal, HashMap::new());

    // hashmap to keep track of the statements before they are sent to the env
    let mut env_variables_map: HashMap<ProtocolType, HashMap<String, String>> = HashMap::new();
    // dbg!(&processes.get("env").unwrap().messages.as_ref().unwrap());
    let real_world = visit_in_order(
        &"env".to_string(),
        &mut processes,
        &ProtocolType::Real,
        &queries_map,
        &mut variables_map,
        &mut branches_variables,
        &mut env_variables_map,
    );
    let mut result_string = format!("{}", real_world.messages.as_ref().unwrap());
    result_string = output_cleaner(result_string);
    fs::write("output_sequence_diagram.txt", result_string)?;
    println!("----------------------------------");
    let ideal_world = visit_in_order(
        &"env".to_string(),
        &mut processes,
        &ProtocolType::Ideal,
        &queries_map,
        &mut variables_map,
        &mut branches_variables,
        &mut env_variables_map,
    );
    let sim_process = insert_branches(processes.remove("sim").unwrap(), &mut branches_variables, &mut env_variables_map);
    let mut sim_string = format!(
        "{}",
        sim_process.messages.as_ref().unwrap()
    );
    for variable in variables_map.keys() {
        let var_string = "=".to_string() + variable;
        let mut new_string = variables_map[variable].clone();
        new_string.insert_str(0, "=");
        new_string = new_string.replace("=(", "(=");
        sim_string = sim_string.replace(&var_string, &new_string);
    }
    // this is the merge between real and ideal world in the env_variables_map
    let mut sim_variables_map: HashMap<String, String> = HashMap::new();
    for (vars, value) in env_variables_map.get(&ProtocolType::Real).unwrap() {
        for (ideal_var, ideal_value) in env_variables_map.get(&ProtocolType::Ideal).unwrap() {
            if value == ideal_value && ideal_var != vars && !sim_variables_map.contains_key(vars){
                sim_variables_map.insert(ideal_var.clone(), vars.clone());
            }
        }
    }
    let mut result_string = format!("{}", sim_process.messages.as_ref().unwrap());
    for (value, replacement) in sim_variables_map {
        if value.starts_with("x"){
            result_string = result_string.replace(&value, &replacement);
        }
    }
    result_string = result_string.replace("\n\n", "\n");
    println!("{}", &result_string);
    if SEQUENCE_DIAGRAM_MODE {
        result_string = output_cleaner(result_string);
    }
    fs::write("output_sequence_diagram_ideal.txt", result_string)?;
    // dbg!(&real_world.messages);
    Ok(())
}
