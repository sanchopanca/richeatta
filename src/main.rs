use clap::{Parser, ValueEnum};

use std::io;

mod memory;

#[derive(Clone, Debug, PartialEq, Eq, ValueEnum)]
enum Mode {
    KnownValue,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// PID of the process to inspect
    #[clap(short, long, value_parser)]
    pid: i32,

    #[arg(value_enum, default_value_t = Mode::KnownValue)]
    mode: Mode,
}

fn main() {
    let args = Args::parse();

    if args.mode == Mode::KnownValue {
        known_value_search(args.pid);
    }
}

fn known_value_search(pid: i32) {
    let process = memory::Process::new(pid);

    let mut search = None;

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
                search = Some(process.search_known_value(value));
                if let Some(search) = &search {
                    println!("{} candidates found", search.count());
                }
            }
            "refine" => {
                if let Some(search) = &mut search {
                    let value = command[1].parse::<i32>().unwrap();
                    search.refine(value);
                    println!("{} candidates found", search.count());
                } else {
                    println!("No search in progress");
                    continue;
                }
            }
            "modify" => {
                if let Some(search) = &search {
                    let value = command[1].parse::<i32>().unwrap();
                    search.modify(value);
                } else {
                    println!("No search in progress");
                    continue;
                }
            }
            "exit" | "quit" | "q" => break,
            _ => println!("Unknown command"),
        }
    }
}
