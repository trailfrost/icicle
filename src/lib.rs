mod args;
#[cfg(test)]
mod tests;

use core::fmt;
use std::{env, error::Error, str::FromStr};

use args::Args;

#[derive(Debug, Clone)]
/// reasons for a help screen to be triggered.
pub enum HelpReason {
    /// user asked for help with `--help`.
    UserAsked,
    /// command lacks an action.
    MissingAction,
    /// required option missing from arguments.
    MissingOption(CLIOption),
    /// required positional argument missing, given start and end indexes.
    MissingArgument(usize, usize),
}

#[derive(Debug, Clone)]
/// reasons that running a command might've failed. this is different from `HelpReason` because it's the return type of the running `command`.
pub enum CommandError {
    /// required option is missing from arguments.
    MissingOption(CLIOption),
    /// required positional argument, given start and end indexes.
    MissingArgument(usize, usize),
}

impl fmt::Display for CommandError {
    /// formats a command.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingOption(option) => write!(f, "Missing option: {:#?}", option),
            Self::MissingArgument(start, end) => {
                write!(f, "Missing arguments from {start} to {end}")
            }
        }
    }
}

impl Error for CommandError {}

#[derive(Debug, Clone)]
/// a command line option (--example, -e).
pub struct CLIOption {
    /// all aliases for this option.
    pub names: Vec<String>,
    /// short description of the option.
    pub desc: String,
    /// whether this option is required.
    pub required: bool,
}

#[derive(Debug, Clone)]
/// a command line positional argument.
pub struct CLIArgument {
    /// short description of the argument.
    pub desc: String,
    /// whether this argument is required.
    pub required: bool,
    /// whether this argument captures multiple values.
    pub array: bool,
}

/// represents a cli command.
pub struct Command {
    /// all aliases for the command.
    names: Vec<String>,
    /// function run when the command is executed.
    action: Option<Box<dyn Fn(Args) -> Result<(), Box<dyn Error + Send + Sync>>>>,
    /// function run to show help screen.
    help: Option<Box<dyn Fn(HelpReason, &Command, Args)>>,
    /// optional short description of the command.
    desc: Option<String>,
    /// subcommands of this command.
    children: Vec<Command>,
    /// options available to this command.
    options: Vec<CLIOption>,
    /// positional arguments for this command.
    arguments: Vec<CLIArgument>,
}

impl Command {
    /// creates a new command with a given name.
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

    /// sets the action to run for the command.
    pub fn action<T: Fn(Args) -> Result<(), Box<dyn Error + Send + Sync>> + 'static>(
        &mut self,
        action: T,
    ) -> &mut Self {
        self.action = Some(Box::new(action));
        self
    }

    /// sets the help action for the command.
    pub fn help<T: Fn(HelpReason, &Command, Args) -> () + 'static>(
        &mut self,
        action: T,
    ) -> &mut Self {
        self.help = Some(Box::new(action));
        self
    }

    /// sets the description of the command.
    pub fn desc(&mut self, desc: &str) -> &mut Self {
        self.desc = Some(desc.to_string());
        self
    }

    /// adds an alias to the command.
    pub fn alias(&mut self, alias: &str) -> &mut Self {
        self.names.push(alias.to_string());
        self
    }

    /// adds a required option with names and description.
    pub fn option(&mut self, names: &str, desc: &str) -> &mut Self {
        let split = names.split(",");

        self.options.push(CLIOption {
            names: split.map(|a| a.trim().to_string()).collect(),
            desc: desc.to_string(),
            required: true,
        });
        self
    }

    /// adds a required positional argument with description.
    pub fn argument(&mut self, desc: &str) -> &mut Self {
        self.arguments.push(CLIArgument {
            desc: desc.to_string(),
            required: true,
            array: false,
        });
        self
    }

    /// adds a positional argument that captures multiple values.
    pub fn array_argument(&mut self, desc: &str) -> &mut Self {
        self.arguments.push(CLIArgument {
            desc: desc.to_string(),
            required: false,
            array: true,
        });
        self
    }

    /// adds an optional option with names and description.
    pub fn opt_option(&mut self, names: &str, desc: &str) -> &mut Self {
        let split = names.split(",");

        self.options.push(CLIOption {
            names: split.map(|a| a.trim().to_string()).collect(),
            desc: desc.to_string(),
            required: false,
        });
        self
    }

    /// adds an optional positional argument.
    pub fn opt_argument(&mut self, desc: &str) -> &mut Self {
        self.arguments.push(CLIArgument {
            desc: desc.to_string(),
            required: false,
            array: false,
        });
        self
    }

    /// adds a subcommand to this command.
    pub fn add(&mut self, other: Command) -> &mut Self {
        self.children.push(other);
        self
    }

    /// creates and adds a new subcommand by name.
    pub fn command(&mut self, name: &str) -> &mut Command {
        let command = Command::new(name);
        self.children.push(command);
        self.children.last_mut().unwrap()
    }

    /// runs the command with given argument strings.
    pub fn run(&self, args: Vec<String>) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        let (command, args, help_option) = Args::parse(self, args);
        if args.has("--help") {
            let reason = HelpReason::MissingAction;
            match help_option {
                Some(help) => help(reason, command, args),
                None => command.default_help(reason),
            }
            return Ok(());
        }

        // check for required options
        for option in command.options.iter() {
            if !option.required {
                continue;
            }

            let mut found = false;
            for name in option.names.iter() {
                if args.has(name) {
                    found = true;
                    break;
                }
            }

            if !found {
                let reason = HelpReason::MissingOption(option.clone());
                match help_option {
                    Some(help) => help(reason, command, args),
                    None => command.default_help(reason),
                }
                return Err(Box::new(CommandError::MissingOption(option.clone())));
            }
        }

        // check for required arguments
        for (pos, arg) in command.arguments.iter().enumerate() {
            if arg.required && !args.has_at(pos) {
                let end = if arg.array {
                    command.arguments.len()
                } else {
                    pos
                };
                let reason = HelpReason::MissingArgument(pos, end);
                match help_option {
                    Some(help) => help(reason, command, args),
                    None => command.default_help(reason),
                }

                return Err(Box::new(CommandError::MissingArgument(pos, end)));
            }
        }

        match &command.action {
            Some(action) => action(args),
            None => {
                let reason = HelpReason::MissingAction;
                match help_option {
                    Some(help) => help(reason, command, args),
                    None => command.default_help(reason),
                }
                Ok(())
            }
        }
    }

    /// runs the command with argument string slices.
    pub fn run_str(&self, args: Vec<&str>) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        self.run(args.iter().map(|arg| arg.to_string()).collect())
    }

    /// runs the command using environment arguments.
    pub fn run_env(&self) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        self.run(env::args().skip(1).collect())
    }

    /// default help function called on help reasons.
    fn default_help(&self, reason: HelpReason) {
        match &reason {
            HelpReason::MissingAction | HelpReason::UserAsked => {
                println!("{}", self.generate_help());
            }
            HelpReason::MissingArgument(start, end) => {
                eprintln!(
                    "missing argument from positions {} to {}!",
                    start + 1,
                    end + 1
                );
                eprintln!("{}", self.generate_help());
            }
            HelpReason::MissingOption(option) => {
                eprintln!("missing option {}!", option.names.join(" or "));
                eprintln!("{}", self.generate_help())
            }
        }
    }

    /// generates a help screen string.
    pub fn generate_help(&self) -> String {
        let mut builder = String::new();
        builder.push_str(&format!("usage:{}\n", self.generate_usage(" ")));
        builder.push_str(&format!("arguments:\n{}", self.generate_args("\t", "\n")));
        builder.push_str(&format!("options:\n{}", self.generate_opts("\t", "\n")));
        builder.push_str(&format!(
            "commands:\n{}",
            self.generate_sub_commands("\t", "\n")
        ));
        builder
    }

    /// generates a usage string with a prefix.
    pub fn generate_usage(&self, prefix: &str) -> String {
        let mut builder = String::from_str(prefix).unwrap();
        builder.push_str(&self.names.get(0).unwrap());
        if self.options.len() > 0 {
            builder.push_str(" [--options]");
        }
        if self.arguments.len() > 0 {
            builder.push_str(" [<arguments>]");
        }
        if self.children.len() > 0 {
            builder.push_str(" <command>");
        }

        builder
    }

    /// generates arguments string with prefix and separator.
    pub fn generate_args(&self, prefix: &str, separator: &str) -> String {
        let mut builder = String::new();
        for (i, arg) in self.arguments.iter().enumerate() {
            builder.push_str(&format!(
                "{}{}: {}{}{}",
                prefix,
                if arg.array {
                    if i != 0 {
                        "<everything else>".to_string()
                    } else {
                        "all arguments".to_string()
                    }
                } else {
                    format!("#{i}")
                },
                arg.desc,
                if arg.required { " (required)" } else { "" },
                separator
            ));
        }

        builder
    }

    /// generates options string with prefix and separator.
    pub fn generate_opts(&self, prefix: &str, separator: &str) -> String {
        let mut builder = String::new();
        for opt in &self.options {
            builder.push_str(&format!(
                "{}{}: {} ({}){}",
                prefix,
                opt.names.join(", "),
                opt.desc,
                if opt.required {
                    "required"
                } else {
                    "not required"
                },
                separator
            ));
        }

        builder
    }

    /// generates subcommands string with prefix and separator.
    pub fn generate_sub_commands(&self, prefix: &str, separator: &str) -> String {
        let mut builder = String::new();
        for command in &self.children {
            builder.push_str(&format!(
                "{}{}: {}{}",
                prefix,
                command.names.join(", "),
                command
                    .desc
                    .clone()
                    .unwrap_or("(no description)".to_string()),
                separator,
            ));
        }

        builder
    }
}
