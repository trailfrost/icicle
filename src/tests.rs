use crate::{Command, args::Args};

fn dummy_command() -> Command {
    Command {
        names: vec!["root".to_string()],
        children: vec![Command {
            names: vec!["sub".to_string(), "alias".to_string()],
            children: vec![],
            arguments: vec![],
            options: vec![],
            help: None,
            action: None,
            desc: None,
        }],
        arguments: vec![],
        options: vec![],
        help: None,
        action: None,
        desc: None,
    }
}

#[test]
fn test_parse_long_option() {
    let cmd = dummy_command();
    let args = vec!["--verbose=true".to_string(), "--count=42".to_string()];
    let (_, parsed, _) = Args::parse(&cmd, args);

    assert_eq!(parsed.get_string("--verbose").unwrap(), "true");
    assert_eq!(parsed.get::<i32>("--count").unwrap(), 42);
}

#[test]
fn test_parse_short_option() {
    let cmd = dummy_command();
    let args = vec!["-v=true".to_string(), "-f=false".to_string()];
    let (_, parsed, _) = Args::parse(&cmd, args);

    assert_eq!(parsed.get_string("v").unwrap(), "true");
    assert_eq!(parsed.get_string("f").unwrap(), "false");
}

#[test]
fn test_positional_arguments() {
    let cmd = dummy_command();
    let args = vec![
        "sub".to_string(),
        "file1.txt".to_string(),
        "file2.txt".to_string(),
    ];
    let (_, parsed, _) = Args::parse(&cmd, args);

    assert_eq!(parsed.at_string(0).unwrap(), "file1.txt");
    assert_eq!(parsed.at_string(1).unwrap(), "file2.txt");
}

#[test]
fn test_command_alias() {
    let cmd = dummy_command();
    let args = vec!["alias".to_string()];
    let (found_cmd, _, _) = Args::parse(&cmd, args);

    assert_eq!(found_cmd.names[0], "sub");
}

#[test]
fn test_has_and_get() {
    let cmd = dummy_command();
    let args = vec!["--enable=true".to_string(), "-d=false".to_string()];
    let (_, parsed, _) = Args::parse(&cmd, args);

    assert!(parsed.has("--enable"));
    assert!(parsed.has("d"));
    assert!(!parsed.has("nonexistent"));

    assert_eq!(parsed.get::<bool>("--enable").unwrap(), true);
    assert_eq!(parsed.get::<bool>("d").unwrap(), false);
}

#[test]
fn test_range_and_iter() {
    let cmd = dummy_command();
    let args = vec![
        "sub".to_string(),
        "1".to_string(),
        "2".to_string(),
        "3".to_string(),
    ];
    let (_, parsed, _) = Args::parse(&cmd, args);

    let range: Vec<i32> = parsed.range(0..3).unwrap();
    assert_eq!(range, vec![1, 2, 3]);

    let strings: Vec<&String> = parsed.range_string(0..3).unwrap();
    assert_eq!(strings, vec!["1", "2", "3"]);

    let collected: Vec<&String> = parsed.iter().collect();
    assert_eq!(collected, vec!["1", "2", "3"]);
}

#[test]
fn test_get_or_and_has_or() {
    let cmd = dummy_command();
    let args = vec!["--primary=10".to_string()];
    let (_, parsed, _) = Args::parse(&cmd, args);

    assert!(parsed.has_or("--primary", "--secondary"));
    assert_eq!(
        parsed.get_or::<i32>("--primary", "--secondary").unwrap(),
        10
    );
}

#[test]
fn test_at_and_out_of_bounds() {
    let cmd = dummy_command();
    let args = vec!["sub".to_string(), "value".to_string()];
    let (_, parsed, _) = Args::parse(&cmd, args);

    assert_eq!(parsed.at_string(0).unwrap(), "value");
    assert!(parsed.at::<String>(1).is_none());
}

#[test]
fn test_command_creation() {
    let cmd = Command::new("test");
    assert_eq!(cmd.names[0], "test");
    assert!(cmd.children.is_empty());
    assert!(cmd.options.is_empty());
    assert!(cmd.arguments.is_empty());
}

#[test]
fn test_add_alias() {
    let mut cmd = Command::new("test");
    cmd.alias("alias1");
    cmd.alias("alias2");
    assert!(cmd.names.contains(&"alias1".to_string()));
    assert!(cmd.names.contains(&"alias2".to_string()));
}

#[test]
fn test_add_option_and_argument() {
    let mut cmd = Command::new("test");
    cmd.option("-o, --option", "an option")
        .argument("an argument");
    assert_eq!(cmd.options.len(), 1);
    assert_eq!(cmd.arguments.len(), 1);
    assert_eq!(
        cmd.options[0].names,
        vec!["-o".to_string(), "--option".to_string()]
    );
    assert!(cmd.options[0].required);
    assert!(cmd.arguments[0].required);
}

#[test]
fn test_add_optional_option_and_argument() {
    let mut cmd = Command::new("test");
    cmd.opt_option("-o, --optional", "optional option")
        .opt_argument("optional argument");
    assert_eq!(cmd.options.len(), 1);
    assert_eq!(cmd.arguments.len(), 1);
    assert!(!cmd.options[0].required);
    assert!(!cmd.arguments[0].required);
}

#[test]
fn test_add_subcommand() {
    let mut parent = Command::new("parent");
    parent.command("child").desc("child command");
    assert_eq!(parent.children.len(), 1);
    assert_eq!(parent.children[0].names[0], "child");
    assert_eq!(parent.children[0].desc.as_ref().unwrap(), "child command");
}

#[test]
fn test_generate_usage() {
    let mut cmd = Command::new("app");
    cmd.option("-v, --verbose", "verbose mode")
        .argument("filename");
    let usage = cmd.generate_usage(" ");
    assert!(usage.contains("[--options]"));
    assert!(usage.contains("[<arguments>]"));
}

#[test]
fn test_generate_help_sections() {
    let mut cmd = Command::new("app");
    cmd.option("-v, --verbose", "verbose mode")
        .argument("filename")
        .command("sub");
    let help = cmd.generate_help();
    assert!(help.contains("usage:"));
    assert!(help.contains("arguments:"));
    assert!(help.contains("options:"));
    assert!(help.contains("commands:"));
}

#[test]
fn test_args_parse_long_option() {
    let cmd = Command::new("app");
    let (_, args, _) = Args::parse(&cmd, vec!["--flag=true".into()]);
    assert!(args.has("--flag"));
    assert_eq!(args.get_string("--flag").unwrap(), "true");
}

#[test]
fn test_args_parse_short_option() {
    let cmd = Command::new("app");
    let (_, args, _) = Args::parse(&cmd, vec!["-f=true".into()]);
    assert!(args.has("f"));
    assert_eq!(args.get_string("f").unwrap(), "true");
}

#[test]
fn test_args_positional_arguments() {
    let mut cmd = Command::new("test");
    cmd.command("run"); // add dummy subcommand

    let args = Args::parse_str(&cmd, vec!["run", "input.txt", "output.txt"]).1;

    assert_eq!(args.at_string(0), Some(&"input.txt".to_string()));
    assert_eq!(args.at_string(1), Some(&"output.txt".to_string()));
}

#[test]
fn test_args_get_or_methods() {
    let cmd = Command::new("app");
    let (_, args, _) = Args::parse(&cmd, vec!["--primary=42".into()]);
    assert_eq!(args.get_or::<i32>("--primary", "--backup"), Some(42));
    assert_eq!(args.get_string_or("--primary", "--backup").unwrap(), "42");
}

#[test]
fn test_args_range() {
    let cmd = Command::new("app");
    let (_, args, _) = Args::parse(&cmd, vec!["1".into(), "2".into(), "3".into()]);
    let range: Vec<i32> = args.range(0..3).unwrap();
    assert_eq!(range, vec![1, 2, 3]);
}
