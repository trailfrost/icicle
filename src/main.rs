use icicle::Command;

fn main() {
    let mut program = Command::new("human");

    program.action(|args| {
        for arg in args.iter() {
            println!("Hello, {}!", arg);
        }
        0
    });

    program
        .command("count")
        .option("-x, --x", "First number")
        .option("-y, --y", "Second number")
        .action(|args| {
            let x = args.get("x").unwrap_or(1);
            let y = args.get("y").unwrap_or(2);
            println!("{x} + {y} = {}", x + y);
            0
        });

    program.run_env();
}
