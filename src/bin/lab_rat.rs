use std::io;
use procfs::process::Process;

fn main() {
    let me = Process::myself().unwrap();
    println!("My PID is {}", me.pid);
    let mut x = Box::new(424242);
    println!("{} at address {:p}", x, x);
    println!("I'm a lab rat");
    for line in io::stdin().lines() {
        let line =  line.unwrap();
        if line == "+" {
            *x += 1;
        }
    }
    println!("{} at address {:p}", x, x);
}