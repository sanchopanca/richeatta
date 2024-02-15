use std::{io, process};

fn main() {
    let pid = process::id();
    println!("PID: {}", pid);

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
