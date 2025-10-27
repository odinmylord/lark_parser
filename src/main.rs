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
    let json_data = fs::read_to_string("result.json")?;
    let data = serde_json::Deserializer::from_str(&json_data);
    let data: HashMap<String, Vec<ParserNode>> = parser::data_parser(data)?;
    let mut processes: HashMap<String, Process> = HashMap::new();
    for (query_type, values) in queries_list.into_iter() {
        match query_type.as_str() {
            "realw" => {
                queries_map.insert(ProtocolType::Real, values);
            }
            "idealw" => {
                queries_map.insert(ProtocolType::Ideal, values);
            }
            _ => {
                ()
            }
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
        &processes,
        &ProtocolType::Real,
        &queries_map,
    );
    let mut result_string = format!("{}", real_world.messages.as_ref().unwrap());
    result_string = output_cleaner(result_string);
    fs::write("output_sequence_diagram.txt", result_string)?;
    // dbg!(&real_world.messages);
    Ok(())
}
