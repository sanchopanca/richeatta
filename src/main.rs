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
    let mut agent = memory::Agent::new(args.pid);
    agent.search(424242, true);
    println!("Found {} candidates", agent.count());
    for _ in io::stdin().lines() {}
    agent.search(424242, false);
    println!("Found {} candidates", agent.count());
    agent.modify(999999);
}
