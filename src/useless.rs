    // pub fn external_viewer(process: Process) -> Process {
    //     // create a copy of the process containing only the messages that are sent or received by the environment
    //     // and any branching nodes
    //     let new_messages = match process.messages {
    //         None => None,
    //         Some(node) => {
    //             let new_node = Node::new(
    //                 "env".to_string(),
    //                 "env".to_string(),
    //                 "".to_string(),
    //                 Direction::In,
    //                 None,
    //             );
    //             Some(Box::new(external_viewer_rec(node, &Box::new(Message::Node(new_node)))))
    //         }
    //     };
    //     Process::new(process.process_name, new_messages)
    // }
// 
    // fn external_viewer_rec(node: Box<Message>, new_node: &Box<Message>) {
    //     match *node {
    //         Message::Node(n) => {  
    //             if n.send_channel == "env" || n.recv_channel == "env" {
    //                 match new_node.as_mut() {
    //                     Message::Node(new_n) => {
    //                         let new_message = Node::new(
    //                             n.send_channel.clone(),
    //                             n.recv_channel.clone(),
    //                             n.statement.clone(),
    //                             n.direction,
    //                             None,
    //                         );
    //                         new_n.next = Some(Box::new(Message::Node(new_message.clone())));
    //                         match n.next {
    //                             None => (),
    //                             Some(next_node) => external_viewer_rec(next_node, Box::new(Message::Node(new_n)))
    //                         }
    //                     },
    //                     Message::BranchingNode(mut new_n) => {
    //                         let new_message = Node::new(
    //                             n.send_channel.clone(),
    //                             n.recv_channel.clone(),
    //                             n.statement.clone(),
    //                             n.direction,
    //                             None,
    //                         );
    //                         new_n.if_branch = Some(Box::new(Message::Node(new_message.clone())));
    //                         new_n.else_branch = Some(Box::new(Message::Node(new_message.clone())));
    //                         match n.next {
    //                             None => Message::BranchingNode(new_n),
    //                             Some(next_node) => external_viewer_rec(next_node, Box::new(Message::BranchingNode(new_n)))
    //                         }
    //                     }
    //                 }
    //             }
    //             else {
    //                 match n.next {
    //                     None => *new_node,
    //                     Some(next_node) => external_viewer_rec(next_node, new_node)
    //                 }
    //             }
    //         }
    //         Message::BranchingNode(bn) => {
    //             let new_if_branch = match bn.if_branch {
    //                 None => None,
    //                 Some(if_node) => Some(Box::new(external_viewer_rec(if_node, new_node.clone()))),
    //             };
    //             let new_else_branch = match bn.else_branch {
    //                 None => None,
    //                 Some(else_node) => Some(Box::new(external_viewer_rec(else_node, new_node.clone()))),
    //             };
    //             Message::BranchingNode(BranchingNode::new(new_if_branch, new_else_branch, None))
    //         }
    //     }
    // }



pub fn insert_branches(
        process: Process,
        branches: &mut HashMap<ProtocolType, HashMap<String, String>>,
        env_variable: &mut HashMap<ProtocolType, HashMap<String, String>>,
    ) -> Process {
        let mut temp_mapping: HashMap<String, String> = HashMap::new();
        // revert the ideal world env_variable mapping
        temp_mapping = env_variable
            .get(&ProtocolType::Ideal)
            .unwrap()
            .iter()
            .map(|(k, v)| (v.clone(), k.clone()))
            .collect();
        let mut branches_statements: HashMap<String, String> = HashMap::new();
        for (additional_string, statement) in branches.get(&ProtocolType::Real).unwrap() {
            for (process_statem, env_statem) in env_variable.get(&ProtocolType::Real).unwrap() {
                if statement == process_statem {
                    branches_statements.insert(
                        additional_string.clone(),
                        temp_mapping.get(env_statem).unwrap().clone(),
                    );
                }
            }
        }
        dbg!(&branches_statements);
        let new_head = insert_branches_rec(
            process.messages.unwrap(),
            &branches_statements,
            "".to_string(),
        );
        Process {
            process_name: process.process_name,
            messages: Some(new_head.unwrap()),
        }
    }

    fn insert_branches_rec(
        node: Box<Message>,
        branches_statements: &HashMap<String, String>,
        additional_string: String,
    ) -> Option<Box<Message>> {
        match *node {
            Message::Node(mut n) => match n.next {
                None => Some(Box::new(Message::Node(n))),
                Some(next_node) => match next_node.as_ref() {
                    Message::Node(next_n) => {
                        println!("Checking next node with statement: {}", n.statement.clone());
                        if next_n.send_channel == "a" && next_n.direction == Direction::In && 
                            branches_statements
                            .values()
                            .any(|val| val == &next_n.statement)
                        {
                            println!("inserting node");
                            let additional_string_if = additional_string.clone() + "_if";
                            let additional_string_else = additional_string.clone() + "_else";
                            let new_next_node_if = insert_branches_rec(next_node.clone(), branches_statements, additional_string_if);
                            let new_next_node_else = insert_branches_rec(next_node.clone(), branches_statements, additional_string_else);
                            let new_branching_node = BranchingNode::new(
                                new_next_node_if,
                                new_next_node_else,
                                Some(next_n.statement.clone()),
                            );
                            Some(Box::new(Message::BranchingNode(new_branching_node)))
                        } else {
                            n.next = insert_branches_rec(next_node, branches_statements, additional_string);
                            Some(Box::new(
                                Message::Node(
                                    n
                                ))
                            )
                        }
                    }
                    Message::BranchingNode(_) => {
                        println!("Checking next branchingnode with statement: {}", n.statement.clone());
                        n.next = insert_branches_rec(next_node, branches_statements, additional_string);
                        Some(Box::new(
                            Message::Node(
                                n
                            ))
                        )
                    }
                },
            },
            Message::BranchingNode(bn) => {
                println!("Checking branchingnode with statement");
                let if_branch = match bn.if_branch {
                    None => None,
                    Some(_) => {
                        let additional_string_if = additional_string.clone() + "_if";
                        let new_if_branch = match bn.if_branch {
                            None => None,
                            Some(if_node) => insert_branches_rec(if_node, branches_statements, additional_string_if),
                        };
                        new_if_branch
                    }
                };
                let else_branch = match bn.else_branch {
                    None => None,
                    Some(_) => {
                        let additional_string_else = additional_string.clone() + "_else";
                        let new_else_branch = match bn.else_branch {
                            None => None,
                            Some(else_node) => insert_branches_rec(else_node, branches_statements,additional_string_else),
                        };
                        new_else_branch
                    }
                };
                Some(Box::new(Message::BranchingNode(BranchingNode::new(
                    if_branch,
                    else_branch,
                    bn.statement.clone(),
                ))))
            }
        }
    }