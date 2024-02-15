use std::{
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
};

use richeatta::memory::Agent;

#[test]
fn test_modify_lab_rat_memory() {
    let mut lab_rat = Command::new("cargo")
        .args(["run", "--bin", "lab_rat"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute lab_rat process");

    let stdout = lab_rat.stdout.take().expect("Failed to capture stdout");
    let mut reader = BufReader::new(stdout);

    let mut line = String::new();
    // reading PID
    reader
        .read_line(&mut line)
        .expect("Failed to read line from stdout");
    print!("Lab Rat says: {}", line);

    let pid = line
        .split_ascii_whitespace()
        .last()
        .unwrap()
        .parse::<i32>()
        .expect("Failed to parse PID");
    assert_eq!(pid, pid);

    let mut agent = Agent::new(pid);
    line.clear();
    // address and value
    reader
        .read_line(&mut line)
        .expect("Failed to read line from stdout");
    print!("Lab Rat says: {}", line);

    agent.search(12345, true);

    assert!(agent.count() > 0);

    let mut stdin = lab_rat.stdin.take().expect("Failed to capture stdin");
    writeln!(stdin, "modify").expect("Failed to write to stdin");
    line.clear();
    // new value
    reader
        .read_line(&mut line)
        .expect("Failed to read line from stdout");
    print!("Lab Rat says: {}", line);

    agent.search(54321, false);

    assert_eq!(agent.count(), 1);

    agent.modify(424242);

    writeln!(stdin, "print").expect("Failed to write to stdin");

    line.clear();
    // new value after our modification
    reader
        .read_line(&mut line)
        .expect("Failed to read line from stdout");
    print!("Lab Rat says: {}", line);
    let value = line.trim().parse::<i32>().expect("Failed to parse value");

    assert_eq!(value, 424242);

    writeln!(stdin, "exit").expect("Failed to write to stdin");

    let _ = lab_rat.wait().expect("Failed to wait on lab_rat");
}
