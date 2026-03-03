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

use crate::tree::extraction::visit_in_order;

#[derive(EnumStringify, Hash, PartialEq, Eq, Debug)]
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
    let json_data = fs::read_to_string("result.json")?;
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
    // dbg!(&processes.get("env").unwrap().messages.as_ref().unwrap());
    let real_world = visit_in_order(
        &"env".to_string(),
        &mut processes,
        &ProtocolType::Real,
        &queries_map,
        &mut variables_map,
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
    );
    let mut sim_string = format!("{}", processes.get("sim").unwrap().messages.as_ref().unwrap());
    for variable in variables_map.keys() {
        let var_string = "=".to_string() + variable;
        let mut new_string = variables_map[variable].clone();
        new_string.insert_str(0, "=");
        new_string = new_string.replace("=(", "(=");
        sim_string = sim_string.replace(&var_string, &new_string);
    }
    print!("sim string: {}", sim_string);
    let mut result_string = format!("{}", ideal_world.messages.as_ref().unwrap());
    result_string = output_cleaner(result_string);
    fs::write("output_sequence_diagram_ideal.txt", result_string)?;
    // dbg!(&real_world.messages);
    Ok(())
}
