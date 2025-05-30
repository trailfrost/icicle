use std::{collections::HashMap, str::FromStr};

use crate::Command;

pub struct Args {
    pub opts: HashMap<String, String>,
    pub pos: Vec<String>,
}

impl Args {
    pub fn parse<'a>(command: &'a Command, arguments: Vec<String>) -> (&'a Command, Args) {
        let mut current_command = command;
        let mut parsed_args = Args {
            opts: HashMap::new(),
            pos: Vec::new(),
        };

        let mut found_command: bool = false;
        let mut ignore_options: bool = false;
        for arg in arguments {
            if !ignore_options {
                if arg.starts_with("--") {
                    let split: Vec<&str> = arg.split("=").collect();
                    let name = split.get(0).unwrap();
                    if name.is_empty() {
                        ignore_options = true;
                        continue;
                    }
                    let value = split.get(1).unwrap_or(&"true");

                    parsed_args.opts.insert(name.to_string(), value.to_string());
                } else if arg.starts_with("-") {
                    let split: Vec<&str> = arg.split("=").collect();
                    let chars: Vec<char> = split.get(0).unwrap().chars().collect();
                    let value = split.get(1).unwrap_or(&"true");

                    for char in chars {
                        parsed_args.opts.insert(char.to_string(), value.to_string());
                    }
                }
            }

            if found_command {
                parsed_args.pos.push(arg);
            } else {
                let mut found = false;
                'child_loop: for command in &current_command.children {
                    for alias in &command.names {
                        if *arg == *alias {
                            current_command = command;
                            found = true;
                            break 'child_loop;
                        }
                    }
                }

                found_command = !found;
            }
        }

        (current_command, parsed_args)
    }

    pub fn parse_str<'a>(command: &'a Command, arguments: Vec<&str>) -> (&'a Command, Args) {
        Self::parse(
            command,
            arguments.iter().map(|arg| arg.to_string()).collect(),
        )
    }

    pub fn new(arguments: Vec<String>) -> Args {
        let (_, arguments) = Self::parse(&Command::new(""), arguments);
        arguments
    }

    pub fn new_str(arguments: Vec<&str>) -> Args {
        let (_, arguments) = Self::parse_str(&Command::new(""), arguments);
        arguments
    }

    pub fn get<T>(&self, name: &str) -> Option<T>
    where
        T: FromStr,
    {
        match self.opts.get(name)?.parse::<T>() {
            Ok(val) => Some(val),
            Err(_) => None,
        }
    }

    pub fn get_or<T>(&self, name: &str, other: &str) -> Option<T>
    where
        T: FromStr,
    {
        if self.opts.contains_key(name) {
            self.get(name)
        } else {
            self.get(other)
        }
    }

    pub fn get_string(&self, name: &str) -> Option<&String> {
        self.opts.get(name)
    }

    pub fn get_string_or(&self, name: &str, other: &str) -> Option<&String> {
        if self.opts.contains_key(name) {
            self.opts.get(name)
        } else {
            self.opts.get(other)
        }
    }

    pub fn at<T>(&self, pos: usize) -> Option<T>
    where
        T: FromStr,
    {
        match self.pos.get(pos)?.parse::<T>() {
            Ok(val) => Some(val),
            Err(_) => None,
        }
    }

    pub fn at_string(&self, pos: usize) -> Option<&String> {
        self.pos.get(pos)
    }
}
