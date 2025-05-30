mod args;
#[cfg(test)]
mod tests;

use args::Args;

/// A command line option (--example, -e).
pub struct CLIOption {
    names: Vec<String>,
    desc: String,
}

/// Represents a CLI command.
pub struct Command {
    /// A list of aliases for the command.
    names: Vec<String>,
    /// The function to run when running this command.
    action: Option<Box<dyn Fn(Args) -> i32>>,
    /// The function to run when a help screen should be shown.
    help: Option<Box<dyn Fn(Args) -> ()>>,
    /// A short description of your command.
    desc: Option<String>,
    /// All subcommands inside this command.
    children: Vec<Command>,
    /// All obligatory options.
    options: Vec<CLIOption>,
    /// All obligatory positional arguments.
    arguments: Vec<String>,
}

impl Command {
    /// Creates a new command.
    pub fn new(name: &str) -> Command {
        Command {
            names: vec![name.to_string()],
            desc: None,
            children: Vec::new(),
            options: Vec::new(),
            arguments: Vec::new(),
            action: None,
            help: None,
        }
    }

    /// Sets the action for a command.
    pub fn action<T: Fn(Args) -> i32 + 'static>(&mut self, action: T) -> &Self {
        self.action = Some(Box::new(action));
        self
    }

    /// Sets the help action for a command.
    pub fn help<T: Fn(Args) -> () + 'static>(&mut self, action: T) -> &Self {
        self.help = Some(Box::new(action));
        self
    }

    /// Sets the description for a command.
    pub fn desc(&mut self, desc: &str) -> &Self {
        self.desc = Some(desc.to_string());
        self
    }

    /// Adds an alias to the command.
    pub fn alias(&mut self, alias: &str) -> &Self {
        self.names.push(alias.to_string());
        self
    }

    /// Adds a required option to the command.
    pub fn option(&mut self, names: &str, desc: &str) -> &Self {
        let split = names.split(",");
        let option = CLIOption {
            names: split.map(|a| a.trim().to_string()).collect(),
            desc: desc.to_string(),
        };

        self.options.push(option);
        self
    }

    /// Adds a required argument to the command.
    pub fn argument(&mut self, desc: &str) -> &Self {
        self.arguments.push(desc.to_string());
        self
    }

    /// Adds a sub command to the command.
    pub fn r#use(&mut self, other: Command) -> &Self {
        self.children.push(other);
        self
    }
}
