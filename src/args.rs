use std::{collections::HashMap, str::FromStr};

pub struct Args {
    pub opts: HashMap<String, String>,
    pub args: Vec<String>,
}

impl Args {
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
        match self.args.get(pos)?.parse::<T>() {
            Ok(val) => Some(val),
            Err(_) => None,
        }
    }

    pub fn at_string(&self, pos: usize) -> Option<&String> {
        self.args.get(pos)
    }
}
