use std::io;

fn main() {
    let x = 424242;
    println!("I'm a lab rat");
    for line in io::stdin().lines() {
        print!("{}", line.unwrap());
    }
    println!("{}", x);
}