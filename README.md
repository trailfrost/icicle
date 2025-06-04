# Icicle

A command line argument parser for Rust. It works like commander.js: you create commands, set up arguments and options for that command, and then set an action, followed by sub commands, etc.

```rust
use icicle::Command;
use std::process;

fn main() {
    let mut program = Command::new("human");

    program
        .command("greet")
        .desc("Greet any amount of people.")
        .array_argument("Names you want to greet.")
        .action(|args| {
            for arg in args.iter() {
                println!("Hello, {}!", arg);
            }

            Ok(())
        });

    program
        .command("add")
        .desc("Add two numbers.")
        .option("-x, --x", "First number")
        .option("-y, --y", "Second number")
        .action(|args| {
            let x = args.get_or::<i32>("-x", "--x").unwrap();
            let y = args.get_or::<i32>("-y", "--y").unwrap();
            println!("{x} + {y} = {}", x + y);

            Ok(())
        })
        .command("infinite")
        .desc("Add any amount of numbers.")
        .array_argument("Numbers you want to add.")
        .action(|args| {
            let mut sum = 0;
            for arg in args.iter() {
                sum += arg.parse::<i32>().unwrap();
            }
            println!("{} = {}", args.join(" + "), sum);

            Ok(())
        });

    if let Err(error) = program.run_env() {
        println!("Error: {error}");
        process::exit(1);
    }
}
```

This creates a program with two commands: `count` and `greet`. As the main program doesn't have an action, running it without arguments will show a help screen.

If you run `human count`, you will need to pass in two options: `-x` and `-y`. Values are separated by an `=`. So, `human count -x=5 -y=5` results in printing out `5 + 5 = 10`.

`count` also has a sub command, called `infinite`, which takes in a variable amount of arguments (that's what `array_argument` does). So, `human count infinite 50 50 25 25` will result in the output `the sum is 150`.

Running `human greet John Amy` will print out:

```
Hello, John!
Hello, Amy!
```

`program.run_env` runs the command with `std::env::args()`.

Icicle auto-generates a `--help` option, which shows a help screen based on what you set up in your command. Running `human greet --help` has this output:

```
usage: greet [<arguments>]
arguments:
    all arguments: Names you want to greet.
options:
commands:
```

## Contributions

Contributions are welcome! Just make sure that for breaking changes or large changes, you open an issue first.

## License

[MIT license](LICENSE.txt)
