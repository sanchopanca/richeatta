use std::{
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
};

use richeatta::memory::Process;

#[test]
fn test_modify_lab_rat_memory() {
    let (pid, mut lab_rat, mut stdin, mut stdout_reader) = launch_lab_rat("known-value");

    let process = Process::new(pid);
    let mut line = String::new();

    // address and value
    stdout_reader
        .read_line(&mut line)
        .expect("Failed to read line from stdout");
    print!("Lab Rat says: {}", line);

    let mut search = process.search_known_value(12345);

    assert!(search.count() > 0);

    send_command(&mut stdin, "modify");

    line.clear();
    // new value
    stdout_reader
        .read_line(&mut line)
        .expect("Failed to read line from stdout");
    print!("Lab Rat says: {}", line);

    search.refine(54321);

    assert_eq!(search.count(), 1);

    search.modify(424242);

    send_command(&mut stdin, "print");

    line.clear();
    // new value after our modification
    stdout_reader
        .read_line(&mut line)
        .expect("Failed to read line from stdout");
    print!("Lab Rat says: {}", line);
    let value = line.trim().parse::<i32>().expect("Failed to parse value");

    assert_eq!(value, 424242);

    send_command(&mut stdin, "exit");

    let _ = lab_rat.wait().expect("Failed to wait on lab_rat");
}

#[test]
#[cfg(target_os = "windows")]
fn test_unknown_value() {
    let (pid, mut lab_rat, mut stdin, mut stdout_reader) = launch_lab_rat("unknown-value");

    let process = Process::new(pid);

    let mut search = process.search_unknown_value::<i8>();

    send_command(&mut stdin, "increase");
    search.value_increased();

    send_command(&mut stdin, "decrease");
    search.value_decreased();

    send_command(&mut stdin, "increase");
    send_command(&mut stdin, "decrease");

    search.value_didnt_change();

    send_command(&mut stdin, "increase");
    send_command(&mut stdin, "increase");
    send_command(&mut stdin, "increase");
    send_command(&mut stdin, "increase");

    search.value_increased();

    send_command(&mut stdin, "decrease");
    search.value_changed();

    send_command(&mut stdin, "increase");
    search.value_changed();

    send_command(&mut stdin, "decrease");
    send_command(&mut stdin, "decrease");
    send_command(&mut stdin, "decrease");
    send_command(&mut stdin, "decrease");

    search.value_decreased();

    send_command(&mut stdin, "decrease");
    send_command(&mut stdin, "increase");

    search.value_didnt_change();

    // the number of refinements isn't really determenistic
    // and is different on my machine and on CI runners
    // so we just refine some more

    send_command(&mut stdin, "decrease");
    send_command(&mut stdin, "decrease");

    search.value_decreased();

    send_command(&mut stdin, "increase");
    send_command(&mut stdin, "increase");

    search.value_increased();

    send_command(&mut stdin, "increase");
    send_command(&mut stdin, "increase");

    search.value_increased();

    assert_eq!(search.count(), 1);

    search.modify(-42i8);

    send_command(&mut stdin, "print");

    let mut line = String::new();
    // new value after our modification
    stdout_reader
        .read_line(&mut line)
        .expect("Failed to read line from stdout");

    let new_value = line.trim().parse::<i8>().unwrap();

    assert_eq!(new_value, -42i8);

    send_command(&mut stdin, "exit");

    let _ = lab_rat.wait().expect("Failed to wait on lab_rat");
}

fn launch_lab_rat(
    command: &str,
) -> (
    i32,
    std::process::Child,
    std::process::ChildStdin,
    BufReader<std::process::ChildStdout>,
) {
    let mut lab_rat = Command::new("cargo")
        .args(["run", "--bin", "lab_rat", "--", command])
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

    let stdin = lab_rat.stdin.take().expect("Failed to capture stdin");

    (pid, lab_rat, stdin, reader)
}

fn send_command(stdin: &mut std::process::ChildStdin, command: &str) {
    writeln!(stdin, "{}", command).expect("Failed to write to stdin");
}
