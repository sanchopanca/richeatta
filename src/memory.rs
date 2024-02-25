use std::mem::size_of;

use self::data_types::Integer;

mod data_types;
#[cfg_attr(target_os = "linux", path = "memory/linux.rs")]
#[cfg_attr(target_os = "windows", path = "memory/windows.rs")]
mod os_specific;

enum State {
    Idle,
    Searching,
}

pub struct Process {
    pid: i32,
    state: State,
    candidates: Vec<usize>,
}

impl Process {
    pub fn new(pid: i32) -> Self {
        Process {
            pid,
            state: State::Idle,
            candidates: Vec::new(),
        }
    }

    pub fn count(&self) -> usize {
        self.candidates.len()
    }

    pub fn modify<T: Integer>(&self, value: T) {
        let address = self.candidates[0];
        os_specific::modify_at_address(self.pid, address, value);
    }

    pub fn search<T: Integer>(&mut self, value: T) {
        self.candidates = os_specific::search_everywhere(self.pid, value);
    }

    // FIXME: If refine is called with a differnet larger type, it can try access memory outside the process memory map
    pub fn refine<T: Integer>(&mut self, new_value: T) {
        self.candidates =
            os_specific::search_among_candidates(self.pid, new_value, &self.candidates);
    }
}

fn first_search<T: Integer>(buffer: &[u8], value: T, base_address: usize) -> Vec<usize> {
    let size = size_of::<T>();
    let mut found = Vec::new();
    for i in (0..buffer.len()).step_by(size) {
        let number = to::<T>(&buffer[i..i + size]);
        if number == value {
            println!("FOUND {} at {:#x}", number, base_address + i);
            found.push(base_address + i);
        }
    }
    found
}

#[cfg(target_os = "linux")]
fn refine_search<T: Integer>(
    buffer: &[u8],
    value: T,
    base_address: usize,
    candidates: &[usize],
) -> Vec<usize> {
    let size = size_of::<T>();
    let mut found = Vec::new();
    for address in candidates {
        let i = address - base_address;
        let number = to::<T>(&buffer[i..i + size]);
        if number == value {
            println!("FOUND {} at {:#x}", number, base_address + i);
            found.push(base_address + i);
        }
    }
    found
}

fn to<T: Integer>(slice: &[u8]) -> T {
    T::from_ne_bytes(slice)
}
