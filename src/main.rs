use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;

use log::debug;

use crate::extraction::Process;
mod parser;
use parser::ParserNode;

#[derive(Deserialize, Debug)]
pub enum Direction {
    In,
    Out,
}

#[derive(Deserialize, Debug)]
pub struct Node {
    send_channel: String,
    recv_channel: String,
    statement: String,
    direction: Direction,
    next: Option<Box<Message>>,
}

impl Node {
    fn new(
        send_channel: String,
        recv_channel: String,
        statement: String,
        direction: Direction,
        next: Option<Box<Message>>,
    ) -> Self {
        Node {
            send_channel,
            recv_channel,
            statement,
            direction,
            next,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct BranchingNode {
    if_branch: Option<Box<Message>>,
    else_branch: Option<Box<Message>>,
}

impl BranchingNode {
    fn new(if_branch: Option<Box<Message>>, else_branch: Option<Box<Message>>) -> Self {
        BranchingNode {
            if_branch,
            else_branch,
        }
    }
}

#[derive(Deserialize, Debug)]
pub enum Message {
    Node(Node),
    BranchingNode(BranchingNode),
}

pub mod extraction {

    use log::debug;
    use std::str::FromStr;

    use crate::{parser::ParserNode, BranchingNode, Direction, Message, Node};

    #[derive(Debug)]
    pub struct Process {
        process_name: String,
        messages: Option<Box<Message>>,
    }

    impl Process {
        pub fn new(process_name: String, messages: Option<Box<Message>>) -> Self {
            Process {
                process_name,
                messages,
            }
        }

        pub fn add_messages(&mut self, messages: &Vec<ParserNode>) {
            let new_messages = Process::create_message_tree(messages);
            self.messages = new_messages;
        }

        fn create_message_tree(messages: &Vec<ParserNode>) -> Option<Box<Message>> {
            dbg!(messages.first());
            let head_option = match messages.first() {
                Some(message) => {
                    let new_message = match &message.message {
                        Some(_) => {
                            let send_channel = match message.send_channel.as_deref() {
                                Some(channel) => String::from_str(channel).unwrap(),
                                None => panic!("send_channel is required in Nodes"),
                            };
                            let recv_channel = match message.receive_channel.as_deref() {
                                Some(channel) => String::from_str(channel).unwrap(),
                                None => panic!("receive_channel is required in Nodes"),
                            };
                            let statement = match message.message.as_deref() {
                                Some(msg) => String::from_str(msg).unwrap(),
                                None => panic!("message is required in Nodes"),
                            };
                            let direction: Direction = match message.direction.as_deref() {
                                Some("in") => Direction::In,
                                Some("out") => Direction::Out,
                                _ => panic!("Invalid direction"),
                            };
                            Box::new(Message::Node(Node::new(
                                send_channel,
                                recv_channel,
                                statement,
                                direction,
                                None,
                            )))
                        }
                        None => {
                            let mut result =
                                Box::new(Message::BranchingNode(BranchingNode::new(None, None)));
                            if let Some(if_branch_message) = &message.if_statem {
                                Process::add_messages_recursively(
                                    if_branch_message,
                                    &mut result,
                                    Some(true),
                                );
                            }
                            if let Some(else_branch_message) = &message.else_statem {
                                Process::add_messages_recursively(
                                    else_branch_message,
                                    &mut result,
                                    Some(false),
                                );
                            }
                            result
                        }
                    };
                    Some(*new_message)
                }
                None => {
                    debug!("No messages to create");
                    None
                }
            };
            if head_option.is_none() {
                debug!("No messages created");
                return None;
            }
            if messages.len() == 1 {
                return Some(Box::new(head_option.unwrap()));
            }
            let mut head = Box::new(head_option.unwrap());
            if let Message::Node(_) = head.as_ref() {
                // BranchingNode already handled in the recursive function
                Process::add_messages_recursively(messages.split_at(1).1, &mut head, None);
            }
            Some(head)
        }

        fn add_messages_recursively(
            messages: &[ParserNode],
            current_node: &mut Box<Message>,
            if_branch: Option<bool>,
        ) {
            match current_node.as_mut() {
                Message::Node(node) => match messages.first() {
                    None => return,
                    Some(message) => {
                        let new_message = match &message.message {
                            Some(_) => {
                                let send_channel = match message.send_channel.as_deref() {
                                    Some(channel) => String::from_str(channel).unwrap(),
                                    None => panic!("send_channel is required in Nodes"),
                                };
                                let recv_channel = match message.receive_channel.as_deref() {
                                    Some(channel) => String::from_str(channel).unwrap(),
                                    None => panic!("receive_channel is required in Nodes"),
                                };
                                let statement = match message.message.as_deref() {
                                    Some(msg) => String::from_str(msg).unwrap(),
                                    None => panic!("messageresult is required in Nodes"),
                                };
                                let direction: Direction = match message.direction.as_deref() {
                                    Some("in") => Direction::In,
                                    Some("out") => Direction::Out,
                                    Some(val) => panic!("Invalid direction {val}"),
                                    None => panic!("direction is required in Nodes"),
                                };
                                Box::new(Message::Node(Node::new(
                                    send_channel,
                                    recv_channel,
                                    statement,
                                    direction,
                                    None,
                                )))
                            }
                            None => {
                                let mut result = Box::new(Message::BranchingNode(
                                    BranchingNode::new(None, None),
                                ));
                                if let Some(if_branch_message) = &message.if_statem {
                                    Process::add_messages_recursively(
                                        if_branch_message,
                                        &mut result,
                                        Some(true),
                                    );
                                }
                                if let Some(else_branch_message) = &message.else_statem {
                                    Process::add_messages_recursively(
                                        else_branch_message,
                                        &mut result,
                                        Some(false),
                                    );
                                }
                                result
                            }
                        };
                        let mut additional_call = false;
                        if let Message::Node(_) = new_message.as_ref() {
                            additional_call = true;
                        }
                        node.next = Some(new_message);
                        if additional_call {
                            Process::add_messages_recursively(
                                messages.split_at(1).1,
                                node.next.as_mut().unwrap(),
                                None,
                            );
                        }
                    }
                },
                Message::BranchingNode(branch_node) => match messages.first() {
                    None => return,
                    Some(message) => {
                        let new_message = match &message.message {
                            Some(_) => {
                                let send_channel = match message.send_channel.as_deref() {
                                    Some(channel) => String::from_str(channel).unwrap(),
                                    None => panic!("send_channel is required in Nodes"),
                                };
                                let recv_channel = match message.receive_channel.as_deref() {
                                    Some(channel) => String::from_str(channel).unwrap(),
                                    None => panic!("receive_channel is required in Nodes"),
                                };
                                let statement = match message.message.as_deref() {
                                    Some(msg) => String::from_str(msg).unwrap(),
                                    None => panic!("message is required in Nodes"),
                                };
                                let direction: Direction = match message.direction.as_deref() {
                                    Some("in") => Direction::In,
                                    Some("out") => Direction::Out,
                                    _ => panic!("Invalid direction"),
                                };
                                Box::new(Message::Node(Node::new(
                                    send_channel,
                                    recv_channel,
                                    statement,
                                    direction,
                                    None,
                                )))
                            }
                            None => {
                                let mut result = Box::new(Message::BranchingNode(
                                    BranchingNode::new(None, None),
                                ));
                                if let Some(if_branch_message) = &message.if_statem {
                                    let mut if_branch_node =
                                        Box::new(Message::BranchingNode(BranchingNode {
                                            if_branch: None,
                                            else_branch: None,
                                        }));
                                    Process::add_messages_recursively(
                                        if_branch_message,
                                        &mut if_branch_node,
                                        Some(true),
                                    );
                                    if let Message::BranchingNode(ref mut branch_node) = *result {
                                        branch_node.if_branch = Some(if_branch_node);
                                    }
                                }
                                if let Some(else_branch_message) = &message.else_statem {
                                    let mut else_branch_node =
                                        Box::new(Message::BranchingNode(BranchingNode {
                                            if_branch: None,
                                            else_branch: None,
                                        }));
                                    Process::add_messages_recursively(
                                        else_branch_message,
                                        &mut else_branch_node,
                                        Some(false),
                                    );
                                    if let Message::BranchingNode(ref mut branch_node) = *result {
                                        branch_node.else_branch = Some(else_branch_node);
                                    }
                                }
                                result
                            }
                        };
                        match if_branch {
                            Some(true) => {
                                let mut additional_call = false;
                                if let Message::Node(_) = new_message.as_ref() {
                                    additional_call = true;
                                }
                                branch_node.if_branch = Some(new_message);
                                if additional_call {
                                    Process::add_messages_recursively(
                                        messages.split_at(1).1,
                                        branch_node.if_branch.as_mut().unwrap(),
                                        None,
                                    );
                                };
                            }
                            Some(false) => {
                                let mut additional_call = false;
                                if let Message::Node(_) = new_message.as_ref() {
                                    additional_call = true;
                                }
                                branch_node.else_branch = Some(new_message);
                                if additional_call {
                                    Process::add_messages_recursively(
                                        messages.split_at(1).1,
                                        branch_node.else_branch.as_mut().unwrap(),
                                        None,
                                    );
                                };
                            }
                            None => {
                                panic!("BranchingNode must be part of an if or else branch");
                            }
                        }
                    }
                },
            }
        }
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json_data = fs::read_to_string("result.json")?;
    let data = serde_json::Deserializer::from_str(&json_data);
    let data: HashMap<String, Vec<ParserNode>> = parser::data_parser(data)?;
    let mut processes: Vec<Process> = Vec::new();
    for (process_name, messages) in data {
        debug!("Processing process: {}", process_name);
        let mut new_process = Process::new(process_name, None);
        new_process.add_messages(&messages);
        processes.push(new_process);
    }
    Ok(())
}
