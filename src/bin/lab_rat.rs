use std::{env, io, process};

fn main() {
    let pid = process::id();
    println!("PID: {}", pid);

    let args = env::args().collect::<Vec<String>>();

    if args.len() == 1 {
        println!("Usage: lab_rat <command>");
        return;
    }

    let command = args[1].as_str();

    match command {
        "known-value" => create_and_modify_one_value(),
        "unknown-value" => create_and_modify_unknown_value(),
        _ => panic!("Unknown command"),
    }
}

fn create_and_modify_one_value() {
    let mut data = Box::new(12345);
    println!("Data address: {:p}, value: {}", &data, data);

    let mut input = String::new();
    loop {
        input.clear();
        io::stdin().read_line(&mut input).unwrap();
        match input.trim() {
            "modify" => {
                *data = 54321;
                println!("Data modified to {}", data);
            }
            "print" => println!("{}", data),
            "exit" => break,
            _ => println!("Unknown command"),
        }
    }
}

fn create_and_modify_unknown_value() {
    let mut data = Box::new(10i8);

    let mut input = String::new();
    loop {
        input.clear();
        io::stdin().read_line(&mut input).unwrap();
        match input.trim() {
            "increase" => {
                *data += 1;
            }
            "decrease" => {
                *data -= 1;
            }
            "print" => println!("{}", data),
            "exit" => break,
            _ => println!("Unknown command"),
        }
    }
}
