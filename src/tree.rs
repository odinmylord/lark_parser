pub mod extraction {
    use crate::{parser::ParserNode, ProtocolType};
    use core::panic;
    use log::debug;
    use std::{
        cell::RefCell,
        collections::HashMap,
        fmt::{Debug, Display},
        rc::Rc,
        str::FromStr,
    };

    static SEQUENCE_DIAGRAM_MODE: bool = false;
    static SEPARATOR: &str = "2";

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum Direction {
        In,
        Out,
    }

    #[derive(Clone, Debug)]
    pub struct Node {
        pub send_channel: String,
        pub recv_channel: String,
        pub statement: String,
        pub direction: Direction,
        pub next: Option<Box<Message>>,
    }

    impl Display for Node {
        // Implement the fmt method to make the node print itself in the sequencediagram.org format
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.direction == Direction::Out {
                match &self.next {
                    None => {
                        if SEQUENCE_DIAGRAM_MODE {
                            f.write_str(&format!(
                                "{}->{}: sent {}",
                                self.send_channel, self.recv_channel, self.statement
                            ))
                        } else {
                            f.write_str(&format!(
                                "out({}{}{},{});\n.",
                                self.send_channel, SEPARATOR, self.recv_channel, self.statement
                            ))
                        }
                    }
                    Some(node) => match node.as_ref() {
                        Message::Node(n) => {
                            if SEQUENCE_DIAGRAM_MODE {
                                f.write_str(&format!(
                                    "{}->{}: sent {}{}",
                                    self.send_channel, self.recv_channel, self.statement, n
                                ))
                            } else {
                                f.write_str(&format!(
                                    "out({}{}{},{});\n{}",
                                    self.send_channel,
                                    SEPARATOR,
                                    self.recv_channel,
                                    self.statement,
                                    n
                                ))
                            }
                        }
                        Message::BranchingNode(bn) => {
                            if let Some(if_branch) = &bn.if_branch {
                                match if_branch.as_ref() {
                                    Message::Node(n) => {
                                        f.write_str(&format!("\nalt if branch:{}\n", n))?;
                                    }
                                    Message::BranchingNode(bn) => {
                                        f.write_str(&format!("\nalt if branch:{}", bn))?;
                                    }
                                }
                            }
                            if let Some(else_branch) = &bn.else_branch {
                                match else_branch.as_ref() {
                                    Message::Node(n) => {
                                        f.write_str(&format!("\nalt else branch:{}\n", n))?;
                                    }
                                    Message::BranchingNode(bn) => {
                                        f.write_str(&format!("\nalt else branch:{}", bn))?;
                                    }
                                }
                            }
                            f.write_str(&format!(
                                "{}->{}: sent {}{}",
                                self.send_channel, self.recv_channel, self.statement, bn
                            ))
                        }
                    },
                }
            } else {
                match self.next.as_ref() {
                    None => {
                        if SEQUENCE_DIAGRAM_MODE {
                            f.write_str(&format!(" received {}", self.statement))
                        } else {
                            f.write_str(&format!(
                                "in({}{}{},{}); let ={} = {} in\n.",
                                self.send_channel, SEPARATOR, self.recv_channel, self.statement, self.statement, self.statement
                            ))
                        }
                    }
                    Some(node) => match node.as_ref() {
                        Message::Node(n) => {
                            if SEQUENCE_DIAGRAM_MODE{
                                f.write_str(&format!(" received {}\n{}", self.statement, n))
                            }
                            else {
                                f.write_str(&format!(
                                    "in({}{}{},{}); let ={} = {} in\n{}",
                                    self.send_channel, SEPARATOR, self.recv_channel, self.statement, self.statement, self.statement, n
                                ))
                            }
                        }
                        Message::BranchingNode(bn) => {
                            f.write_str(&format!(" received {}\n{}", self.statement, bn))
                        }
                    },
                }
            }
        }
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
        fn go_next(self) -> Option<Box<Message>> {
            self.next
        }
    }

    #[derive(Clone, Debug)]
    pub struct BranchingNode {
        pub if_branch: Option<Box<Message>>,
        pub else_branch: Option<Box<Message>>,
        pub statement: Option<String>,
    }

    impl Display for BranchingNode {
        // Implement the fmt method to make the node print itself in the sequencediagram.org format
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if let Some(if_branch) = &self.if_branch {
                match if_branch.as_ref() {
                    Message::Node(n) => {
                        f.write_str(&format!("\nalt if branch:\n{}", n))?;
                    }
                    Message::BranchingNode(bn) => {
                        f.write_str(&format!("\nalt if branch:\n{}", bn))?;
                    }
                }
            }
            if let Some(else_branch) = &self.else_branch {
                match else_branch.as_ref() {
                    Message::Node(n) => {
                        f.write_str(&format!("\nelse else branch:\n{}", n))?;
                    }
                    Message::BranchingNode(bn) => {
                        f.write_str(&format!("\nelse else branch:\n{}", bn))?;
                    }
                }
            }
            f.write_str("\nend")?;
            Ok(())
        }
    }

    impl BranchingNode {
        fn new(
            if_branch: Option<Box<Message>>,
            else_branch: Option<Box<Message>>,
            statement: Option<String>,
        ) -> Self {
            BranchingNode {
                if_branch,
                else_branch,
                statement,
            }
        }

        fn go_if(self) -> Option<Box<Message>> {
            self.if_branch
        }
        fn go_else(self) -> Option<Box<Message>> {
            self.else_branch
        }
    }

    #[derive(Clone, Debug)]
    pub enum Message {
        Node(Node),
        BranchingNode(BranchingNode),
    }

    impl Display for Message {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Message::Node(node) => write!(f, "{}", node),
                Message::BranchingNode(branch_node) => write!(f, "{}", branch_node),
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct Process {
        pub process_name: String,
        pub messages: Option<Box<Message>>,
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
                            let mut result = Box::new(Message::BranchingNode(BranchingNode::new(
                                None, None, None,
                            )));
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
                                    None => panic!("message result is required in Nodes"),
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
                                    BranchingNode::new(None, None, None),
                                ));
                                if let Some(if_branch_message) = &message.if_statem {
                                    Process::add_messages_recursively(
                                        if_branch_message,
                                        &mut result,
                                        Some(true),
                                    );
                                }
                                let next_message = messages.get(1).expect("A message containing the if_branch has to be followed by one containing the else_branch");
                                match &next_message.else_statem {
                                    None => panic!("A message containing the if_branch has to be followed by one containing the else_branch"),
                                    Some(else_branch_message) => {
                                        Process::add_messages_recursively(
                                            else_branch_message,
                                            &mut result,
                                            Some(false),
                                        );
                                    }
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
                        // ParserNodes with if_statem or else_statem do not have message field set
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
                                    BranchingNode::new(None, None, None),
                                ));
                                if let Some(if_branch_message) = &message.if_statem {
                                    Process::add_messages_recursively(
                                        if_branch_message,
                                        &mut result,
                                        Some(true),
                                    );
                                }
                                let next_message = messages.get(1).expect("A message containing the if_branch has to be followed by one containing the else_branch");
                                match &next_message.else_statem {
                                    None => panic!("A message containing the if_branch has to be followed by one containing the else_branch"),
                                    Some(else_branch_message) => {
                                        Process::add_messages_recursively(
                                            else_branch_message,
                                            &mut result,
                                            Some(false),
                                        );
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

        pub fn append_message(&mut self, message: Box<Message>) {
            match self.messages.as_mut() {
                None => {
                    self.messages = Some(message);
                }
                Some(node) => {
                    Process::append_message_recursively(node, message);
                }
            }
        }

        fn append_message_recursively(current_node: &mut Box<Message>, message: Box<Message>) {
            match current_node.as_mut() {
                Message::Node(node) => match node.next.as_mut() {
                    None => node.next = Some(message),
                    Some(next_node) => Process::append_message_recursively(next_node, message),
                },
                Message::BranchingNode(branch_node) => {
                    if let Some(if_branch) = &mut branch_node.if_branch {
                        Process::append_message_recursively(if_branch, message.clone());
                    }
                    if let Some(else_branch) = &mut branch_node.else_branch {
                        Process::append_message_recursively(else_branch, message.clone());
                    }
                }
            }
        }
    }

    pub fn visit_in_order(
        starting_process: &String,
        processes: &mut HashMap<String, Process>,
        protocol: &ProtocolType,
        queries: &HashMap<ProtocolType, HashMap<String, String>>,
        variables_map: &mut HashMap<String, String>,
    ) -> Process {
        if processes.get(starting_process).is_none() {
            panic!("Invalid starting process name");
        }
        // TODO consider case in which the environment starts with a branching node (do not know if possible)
        let mut processes_status: HashMap<String, Rc<RefCell<Option<Box<Message>>>>> =
            HashMap::new();
        for (process_name, status) in processes.iter() {
            processes_status.insert(
                process_name.clone(),
                Rc::new(RefCell::new(status.messages.clone())),
            );
        }
        let first_node = match processes.get(starting_process) {
            None => {
                panic!("Starting process not found in statuses");
            }
            Some(process) => match &process.messages {
                None => {
                    panic!("Starting process has no messages");
                }
                Some(node) => node.clone(),
            },
        };
        if protocol == &ProtocolType::Ideal {
            processes.insert("sim".to_string(), Process::new("sim".to_string(), None));
            debug!("Ideal world");
        } else {
            debug!("Real world");
        }
        update_status(starting_process.clone(), &mut processes_status, false);
        let new_head = visit_in_order_rec(
            first_node,
            &starting_process,
            processes,
            processes_status,
            protocol,
            queries,
            String::from(""),
            9000,
            variables_map,
        );
        Process::new(protocol.to_string(), new_head)
    }

    fn visit_in_order_rec(
        current_node: Box<Message>,
        current_process: &String,
        processes: &mut HashMap<String, Process>,
        mut statuses: HashMap<String, Rc<RefCell<Option<Box<Message>>>>>,
        protocol: &ProtocolType,
        queries: &HashMap<ProtocolType, HashMap<String, String>>,
        additional_string: String,
        mut variable_count: i32,
        variables_map: &mut HashMap<String, String>,
    ) -> Option<Box<Message>> {
        match *current_node {
            Message::Node(mut node) => {
                let mut next_process = match node.direction {
                    Direction::In => queries
                        .get(protocol)
                        .expect("Malformed query mapping dictionary")
                        .get(&node.recv_channel)
                        .expect(&format!(
                            "Channel {} not found in mapping",
                            &node.recv_channel
                        )),
                    Direction::Out => queries
                        .get(protocol)
                        .expect("Malformed query mapping dictionary")
                        .get(&node.recv_channel)
                        .expect(&format!(
                            "Channel {} not found in mapping",
                            &node.recv_channel
                        )),
                };
                let new_tmp_string = next_process.clone() + &additional_string;
                if statuses.get(&new_tmp_string).is_some() {
                    next_process = &new_tmp_string;
                }
                let mut skip_status_update = false;
                let next_node = match statuses.get(next_process) {
                    None => {
                        skip_status_update = true;
                        // We have to fill in the gap with the potential next_node;
                        let options = find_next_process(&node.recv_channel, &mut statuses);
                        variables_map.insert(
                            "x".to_string() + &variable_count.to_string(),
                            node.statement.clone(),
                        );
                        if options.len() == 0 {
                            panic!(
                                "No possible next process found for channel {}",
                                &node.recv_channel
                            );
                        }
                        let next_process_found = &options[0];
                        // dbg!(next_process_found);
                        // dbg!(&node.statement);
                        // get the next node of the next process
                        let next_process_name = match queries.get(protocol) {
                            None => {
                                panic!("Protocol {} not found in mapping", protocol);
                            }
                            Some(mapping) => match mapping.get(next_process_found) {
                                None => {
                                    panic!(
                                        "Channel {} not found in mapping for protocol {}",
                                        next_process_found, protocol
                                    );
                                }
                                Some(process_name) => process_name,
                            },
                        };
                        let next_process_node = match statuses.get(next_process_name) {
                            None => {
                                panic!("Process {} not found in statuses", next_process_name);
                            }
                            Some(node) => match node.borrow().as_ref() {
                                None => {
                                    panic!("Process {} has no messages", next_process_name);
                                }
                                Some(node) => node.clone(),
                            },
                        };
                        let next_node_statem = match next_process_node.as_ref() {
                            Message::Node(n) => n.statement.clone(),
                            Message::BranchingNode(_) => {
                                panic!("There should never be a branching node without getting a message in before")
                            }
                        };
                        // get the statement to insert in the out node by looking at the corresponding in node
                        let out_statem = variables_map
                            .get(&next_node_statem)
                            .unwrap_or(&next_node_statem);
                        // create the new nodes to insert in the tree
                        let out_node = Node::new(
                            node.recv_channel.clone(),
                            next_process_found.clone(),
                            out_statem.clone(),
                            Direction::Out,
                            None,
                        );
                        let in_node = Node::new(
                            node.send_channel.clone(),
                            node.recv_channel.clone(),
                            String::from(format!("x{}", variable_count)),
                            Direction::In,
                            Some(Box::new(Message::Node(out_node.clone()))),
                        );
                        //add the in and out nodes to the processes vector
                        match processes.get_mut(next_process) {
                            None => {
                                panic!("Process {} not found in processes", next_process);
                            }
                            Some(process) => {
                                process.append_message(Box::new(Message::Node(in_node.clone())));
                            }
                        };

                        variable_count += 1;
                        next_process = next_process_name;

                        // node.next = Some(Box::new(Message::Node(in_node)));
                        Some(Box::new(Message::Node(out_node)))
                    }
                    Some(n) => n.borrow().clone(),
                };
                match next_node {
                    None => Some(Box::new(Message::Node(node))),
                    Some(next_node) => {
                        if !skip_status_update {
                            update_status(next_process.clone(), &mut statuses, false);
                        }
                        node.next = visit_in_order_rec(
                            next_node,
                            next_process,
                            processes,
                            statuses.clone(),
                            protocol,
                            queries,
                            additional_string.clone(),
                            variable_count,
                            variables_map,
                        );
                        Some(Box::new(Message::Node(node)))
                    }
                }
            }
            Message::BranchingNode(branching_node) => {
                let if_branch = match branching_node.if_branch {
                    None => None,
                    Some(if_node) => {
                        let if_branch_name = current_process.clone() + "_if";
                        let new_additional_string = additional_string.clone() + "_if";
                        statuses.insert(
                            if_branch_name.clone(),
                            Rc::new(RefCell::new(Some(if_node.clone()))),
                        );
                        update_status(if_branch_name.clone(), &mut statuses, true);
                        let statuses_deep_clone: HashMap<
                            String,
                            Rc<RefCell<Option<Box<Message>>>>,
                        > =
                            HashMap::from_iter(statuses.iter().map(|(k, v)| {
                                (k.clone(), Rc::new(RefCell::new(v.borrow().clone())))
                            }));
                        let res = visit_in_order_rec(
                            if_node,
                            &if_branch_name,
                            processes,
                            statuses_deep_clone,
                            protocol,
                            queries,
                            new_additional_string,
                            variable_count,
                            variables_map,
                        );
                        statuses.remove(&if_branch_name);
                        res
                    }
                };
                let else_branch = match branching_node.else_branch {
                    None => None,
                    Some(else_node) => {
                        let else_branch_name = current_process.clone() + "_else";
                        let new_additional_string = additional_string + "_else";
                        statuses.insert(
                            else_branch_name.clone(),
                            Rc::new(RefCell::new(Some(else_node.clone()))),
                        );
                        update_status(else_branch_name.clone(), &mut statuses, false);
                        let statuses_deep_clone: HashMap<
                            String,
                            Rc<RefCell<Option<Box<Message>>>>,
                        > =
                            HashMap::from_iter(statuses.iter().map(|(k, v)| {
                                (k.clone(), Rc::new(RefCell::new(v.borrow().clone())))
                            }));
                        let res = visit_in_order_rec(
                            else_node,
                            &else_branch_name,
                            processes,
                            statuses_deep_clone,
                            protocol,
                            queries,
                            new_additional_string,
                            variable_count,
                            variables_map,
                        );

                        statuses.remove(&else_branch_name);
                        res
                    }
                };
                Some(Box::new(Message::BranchingNode(BranchingNode::new(
                    if_branch,
                    else_branch,
                    None,
                ))))
            }
        }
    }

    fn find_next_process(
        current_channel: &String,
        statuses: &mut HashMap<String, Rc<RefCell<Option<Box<Message>>>>>,
    ) -> Vec<String> {
        // find the next process to visit based on which process can accept messages from the current one
        let mut possibilities: Vec<String> = vec![];
        for status in statuses.iter() {
            match status.1.borrow().as_ref() {
                None => continue,
                Some(node) => match node.as_ref() {
                    Message::Node(n) => {
                        if n.send_channel == *current_channel {
                            possibilities.push(n.recv_channel.clone());
                        }
                    }
                    Message::BranchingNode(_) => {
                        panic!("TODO: understand how to handle branching nodes here")
                    }
                },
            }
        }
        possibilities
    }

    /// Update the statuses vector by putting the next node of the process with name process_name as the new status of the process.
    fn update_status(
        process_name: String,
        statuses: &mut HashMap<String, Rc<RefCell<Option<Box<Message>>>>>,
        if_branch: bool,
    ) {
        let new_status = match statuses.remove(&process_name) {
            None => {
                panic!("Process not found in statuses");
            }
            Some(node) => node.take(),
        };
        let new_entry = match new_status {
            None => {
                debug!("Process {process_name} has no more messages");
                Rc::new(RefCell::new(None))
            }
            Some(node) => match *node {
                Message::Node(n) => {
                    debug!("Process {process_name} next node is a Node");
                    Rc::new(RefCell::new(n.go_next()))
                }
                Message::BranchingNode(bn) => {
                    debug!("Process {process_name} next node is a BranchingNode");
                    if if_branch {
                        Rc::new(RefCell::new(bn.go_if()))
                    } else {
                        Rc::new(RefCell::new(bn.go_else()))
                    }
                }
            },
        };
        statuses.insert(process_name, new_entry);
    }
}
