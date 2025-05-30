mod args;
#[cfg(test)]
mod tests;

use std::{env, str::FromStr};

use args::Args;

/// Reasons for a help screen to be triggered.
pub enum HelpReason {
    /// The user asked for help by passing a `--help` argument.
    UserAsked,
    /// The command the user tried to run doesn't have an action.
    MissingAction,
    /// An option is missing from the CLI arguments.
    MissingOption(CLIOption),
    /// A positional argument is missing from the CLI arguments.
    MissingArgument(usize, usize),
}

/// A command line option (--example, -e).
pub struct CLIOption {
    /// All aliases for the CLI option.
    pub names: Vec<String>,
    /// A short description of the CLI option.
    pub desc: String,
    /// If this option is required for the command to run.
    pub required: bool,
}

/// A command line positional argument.
pub struct CLIArgument {
    /// A short description of the positional argument.
    pub desc: String,
    /// If this argument is required for the command to run.
    pub required: bool,
}

/// Represents a CLI command.
pub struct Command {
    /// A list of aliases for the command.
    names: Vec<String>,
    /// The function to run when running this command.
    action: Option<Box<dyn Fn(Args) -> i32>>,
    /// The function to run when a help screen should be shown.
    help: Option<Box<dyn Fn(HelpReason, &Command, Args) -> ()>>,
    /// A short description of your command.
    desc: Option<String>,
    /// All subcommands inside this command.
    children: Vec<Command>,
    /// All options in the command.
    options: Vec<CLIOption>,
    /// All positional arguments in the command.
    arguments: Vec<CLIArgument>,
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
    pub fn help<T: Fn(HelpReason, &Command, Args) -> () + 'static>(&mut self, action: T) -> &Self {
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

        self.options.push(CLIOption {
            names: split.map(|a| a.trim().to_string()).collect(),
            desc: desc.to_string(),
            required: true,
        });
        self
    }

    /// Adds a required argument to the command.
    pub fn argument(&mut self, desc: &str) -> &Self {
        self.arguments.push(CLIArgument {
            desc: desc.to_string(),
            required: true,
        });
        self
    }

    /// Adds an optional option to the command.
    pub fn opt_option(&mut self, names: &str, desc: &str) -> &Self {
        let split = names.split(",");

        self.options.push(CLIOption {
            names: split.map(|a| a.trim().to_string()).collect(),
            desc: desc.to_string(),
            required: false,
        });
        self
    }

    /// Adds an optional argument to the command.
    pub fn opt_argument(&mut self, desc: &str) -> &Self {
        self.arguments.push(CLIArgument {
            desc: desc.to_string(),
            required: false,
        });
        self
    }

    /// Adds a sub command to the command.
    pub fn r#use(&mut self, other: Command) -> &Self {
        self.children.push(other);
        self
    }

    /// Creates a new sub command and adds it to the command.
    pub fn command(&mut self, name: &str) -> &Command {
        let command = Command::new(name);
        self.children.push(command);
        self.children.last().unwrap()
    }

    /// Runs the command with the arguments you specify.
    pub fn run(&self, args: Vec<String>) -> i32 {
        let (command, args, help_option) = Args::parse(self, args);
        if args.has("help") {
            let reason = HelpReason::MissingAction;
            match help_option {
                Some(help) => help(reason, command, args),
                None => command.default_help(reason),
            }
            return 0;
        }

        match &command.action {
            Some(action) => action(args),
            None => {
                let reason = HelpReason::MissingAction;
                match help_option {
                    Some(help) => help(reason, command, args),
                    None => command.default_help(reason),
                }
                0
            }
        }
    }

    /// Runs the command with a list of string slices instead of `String`s.
    pub fn run_str(&self, args: Vec<&str>) -> i32 {
        self.run(args.iter().map(|arg| arg.to_string()).collect())
    }

    /// Runs the command with `env::args()`.
    pub fn run_env(&self) -> i32 {
        self.run(env::args().collect())
    }

    /// Default help function.
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

    /// Generates a help screen.
    pub fn generate_help(&self) -> String {
        let mut builder = String::new();
        builder.push_str(&format!("usage:{}", self.generate_usage(" ")));
        builder.push_str(&format!("arguments: {}", self.generate_args("\t", "\n")));
        builder.push_str(&format!("options: {}", self.generate_opts("\t", "\n")));
        builder.push_str(&format!(
            "commands: {}",
            self.generate_sub_commands("\t", "\n")
        ));
        builder
    }

    /// Generates a usage string.
    pub fn generate_usage(&self, prefix: &str) -> String {
        let mut builder = String::from_str(prefix).unwrap();
        builder.push_str(&self.names.get(0).unwrap());
        if self.options.len() > 0 {
            builder.push_str(" [--options]");
        }
        if self.arguments.len() > 0 {
            builder.push_str(" [<arguments>]");
        }

        builder
    }

    /// Generates an arguments string.
    pub fn generate_args(&self, prefix: &str, separator: &str) -> String {
        let mut builder = String::new();
        for (i, arg) in self.arguments.iter().enumerate() {
            builder.push_str(&format!(
                "{}#{}: {} ({}){}",
                prefix,
                i,
                arg.desc,
                if arg.required {
                    "required"
                } else {
                    "not required"
                },
                separator
            ));
        }

        builder
    }

    /// Generates an options string.
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

    /// Generates a sub commands string.
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
