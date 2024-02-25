use clap::Parser;

use std::io;

mod memory;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// PID of the process to inspect
    #[clap(short, long, value_parser)]
    pid: i32,
}

fn main() {
    let args = Args::parse();
    let mut process = memory::Process::new(args.pid);

    let mut input = String::new();
    loop {
        input.clear();
        io::stdin().read_line(&mut input).unwrap();
        let command = input.trim().split_ascii_whitespace().collect::<Vec<&str>>();
        if command.is_empty() {
            continue;
        }
        match command[0] {
            "search" => {
                let value = command[1].parse::<i32>().unwrap();
                process.search(value, true);
                println!("{} candidates found", process.count());
            }
            "refine" => {
                let value = command[1].parse::<i32>().unwrap();
                process.search(value, false);
                println!("{} candidates found", process.count());
            }
            "modify" => {
                let value = command[1].parse::<i32>().unwrap();
                process.modify(value);
            }
            "exit" | "quit" | "q" => break,
            _ => println!("Unknown command"),
        }
    }
}
