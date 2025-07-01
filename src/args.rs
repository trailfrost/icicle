use std::{
    collections::{
        HashMap,
        hash_map::{Iter as MapIter, IterMut as MapIterMut},
    },
    ops::Range,
    slice::{Iter, IterMut},
    str::FromStr,
};

use crate::{Command, HelpReason};

/// stores parsed command line arguments.
pub struct Args {
    /// map of option names to their values.
    pub opts: HashMap<String, String>,
    /// list of positional arguments.
    pub pos: Vec<String>,
}

impl Args {
    /// parses command line arguments for the given command.
    pub fn parse<'a>(
        command: &'a Command,
        arguments: Vec<String>,
    ) -> (
        &'a Command,
        Args,
        Option<&'a Box<dyn Fn(HelpReason, &'a Command, Args)>>,
    ) {
        let mut current_command = command;
        let mut parsed_args = Args {
            opts: HashMap::new(),
            pos: Vec::new(),
        };
        let mut help_fn = None;

        let mut ignore_options = false;

        for arg in arguments {
            // tries to match argument as a subcommand of current_command.
            let mut is_subcommand = false;
            for cmd in &current_command.children {
                if cmd.names.iter().any(|alias| alias == &arg) {
                    current_command = cmd;
                    help_fn = cmd.help.as_ref().or(help_fn);
                    is_subcommand = true;
                    break;
                }
            }

            if is_subcommand {
                // continue parsing after moving into subcommand.
                continue;
            }

            if !ignore_options {
                if arg == "--" {
                    // disables option parsing after '--'.
                    ignore_options = true;
                    continue;
                } else if arg.starts_with("--") {
                    // parses long option with optional value.
                    let split: Vec<&str> = arg.splitn(2, '=').collect();
                    let name = split[0];
                    let value = split.get(1).unwrap_or(&"true");
                    parsed_args.opts.insert(name.to_string(), value.to_string());
                    continue;
                } else if arg.starts_with('-') {
                    // parses one or more short options with optional value.
                    let split: Vec<&str> = arg.splitn(2, '=').collect();
                    let chars: Vec<char> = split[0].chars().skip(1).collect(); // skip leading '-'
                    let value = split.get(1).unwrap_or(&"true");
                    for ch in chars {
                        parsed_args.opts.insert(format!("-{ch}"), value.to_string());
                    }
                    continue;
                }
            }

            // treats argument as a positional argument.
            parsed_args.pos.push(arg);
        }

        (current_command, parsed_args, help_fn)
    }

    /// parses command line arguments from a slice of string slices.
    pub fn parse_str<'a>(
        command: &'a Command,
        arguments: Vec<&str>,
    ) -> (
        &'a Command,
        Args,
        Option<&'a Box<dyn Fn(HelpReason, &'a Command, Args)>>,
    ) {
        Self::parse(
            command,
            arguments.iter().map(|arg| arg.to_string()).collect(),
        )
    }

    /// creates Args from a vector of argument strings with an empty command.
    pub fn new(arguments: Vec<String>) -> Args {
        let (_, arguments, _) = Self::parse(&Command::new(""), arguments);
        arguments
    }

    /// creates Args from a vector of argument string slices with an empty command.
    pub fn new_str(arguments: Vec<&str>) -> Args {
        let (_, arguments, _) = Self::parse_str(&Command::new(""), arguments);
        arguments
    }

    /// checks if an option with the given name exists.
    pub fn has(&self, name: &str) -> bool {
        self.opts.contains_key(name)
    }

    /// checks if either of two options exists.
    pub fn has_or(&self, name: &str, other: &str) -> bool {
        self.opts.contains_key(name) || self.opts.contains_key(other)
    }

    /// checks if there is a positional argument at the given index.
    pub fn has_at(&self, pos: usize) -> bool {
        pos < self.pos.len()
    }

    /// tries to get and parse the option value by name into type T.
    pub fn get<T>(&self, name: &str) -> Option<T>
    where
        T: FromStr,
    {
        match self.opts.get(name)?.parse::<T>() {
            Ok(val) => Some(val),
            Err(_) => None,
        }
    }

    /// tries to get and parse the option value by either name or other into type T.
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

    /// gets the option value as a string reference.
    pub fn get_string(&self, name: &str) -> Option<&String> {
        self.opts.get(name)
    }

    /// gets the option value as string reference for either name or other.
    pub fn get_string_or(&self, name: &str, other: &str) -> Option<&String> {
        if self.opts.contains_key(name) {
            self.opts.get(name)
        } else {
            self.opts.get(other)
        }
    }

    /// tries to get and parse the positional argument at index into type T.
    pub fn at<T>(&self, pos: usize) -> Option<T>
    where
        T: FromStr,
    {
        match self.pos.get(pos)?.parse::<T>() {
            Ok(val) => Some(val),
            Err(_) => None,
        }
    }

    /// gets the positional argument at index as string reference.
    pub fn at_string(&self, pos: usize) -> Option<&String> {
        self.pos.get(pos)
    }

    /// parses a range of positional arguments into a vector of type T.
    pub fn range<T>(&self, range: Range<usize>) -> Result<Vec<T>, String>
    where
        T: FromStr,
        T::Err: ToString,
    {
        let slice = self
            .pos
            .get(range)
            .ok_or_else(|| "index out of bounds".to_string())?;

        slice
            .iter()
            .map(|s| s.parse::<T>().map_err(|e| e.to_string()))
            .collect()
    }

    /// gets a range of positional arguments as string references.
    pub fn range_string(&self, range: Range<usize>) -> Option<Vec<&String>> {
        self.pos.get(range).map(|slice| slice.iter().collect())
    }

    /// returns an iterator over positional arguments.
    pub fn iter(&self) -> Iter<String> {
        self.pos.iter()
    }

    /// returns an iterator over options.
    pub fn iter_opt(&self) -> MapIter<String, String> {
        self.opts.iter()
    }

    /// returns a mutable iterator over positional arguments.
    pub fn iter_mut(&mut self) -> IterMut<String> {
        self.pos.iter_mut()
    }

    /// returns a mutable iterator over options.
    pub fn iter_mut_opt(&mut self) -> MapIterMut<String, String> {
        self.opts.iter_mut()
    }

    /// joins positional arguments.
    pub fn join(&self, separator: &str) -> String {
        self.pos.join(separator)
    }
}

impl IntoIterator for Args {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.pos.into_iter()
    }
}
