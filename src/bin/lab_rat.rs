use procfs::process::Process;
use std::io;

fn main() {
    let me = Process::myself().unwrap();
    println!("My PID is {}", me.pid);
    let mut x = Box::new(424242);
    let y = Box::new(424242);
    println!("x = {} at address {:p}", x, x);
    println!("y = {} at address {:p}", y, y);
    println!("I'm a lab rat");
    for line in io::stdin().lines() {
        let line = line.unwrap();
        if line == "+" {
            *x += 1;
        }
    }
    println!("x = {} at address {:p}", x, x);
    println!("y = {} at address {:p}", y, y);
}
