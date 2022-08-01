use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
   /// PID of the process to inspect
   #[clap(short, long, value_parser)]
   pid: u32,
}

fn main() {
    let args = Args::parse();
    println!("Hello, world! PID is {}", args.pid);
}
