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

            0
        });

    program
        .command("add")
        .desc("Add two numbers.")
        .option("-x, --x", "First number")
        .option("-y, --y", "Second number")
        .action(|args| {
            let x = args.get::<i32>("x").unwrap();
            let y = args.get::<i32>("y").unwrap();
            println!("{x} + {y} = {}", x + y);

            0
        })
        .command("infinite")
        .desc("Add any amount of numbers.")
        .array_argument("Numbers you want to add.")
        .action(|args| {
            let mut sum = 0;
            for arg in args.iter() {
                sum += arg.parse::<i32>().unwrap();
            }
            println!("the sum is {sum}");

            0
        });

    process::exit(program.run_env());
}
