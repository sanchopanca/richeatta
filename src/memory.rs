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

    pub fn modify(&self, value: i32) {
        let address = self.candidates[0];
        os_specific::modify_at_address(self.pid, address, value);
    }

    pub fn search(&mut self, value: i32) {
        self.candidates = os_specific::search_everywhere(self.pid, value);
    }

    pub fn refine(&mut self, new_value: i32) {
        self.candidates =
            os_specific::search_among_candidates(self.pid, new_value, &self.candidates);
    }
}

fn first_search(buffer: &[u8], value: i32, base_address: usize) -> Vec<usize> {
    let mut found = Vec::new();
    for i in (0..buffer.len()).step_by(4) {
        let number = to_i32(&buffer[i..i + 4]);
        if number == value {
            println!("FOUND {} at {:#x}", number, base_address + i);
            found.push(base_address + i);
        }
    }
    found
}

#[cfg(target_os = "linux")]
fn refine_search(
    buffer: &[u8],
    value: i32,
    base_address: usize,
    candidates: &[usize],
) -> Vec<usize> {
    let mut found = Vec::new();
    for address in candidates {
        let i = address - base_address;
        let number = to_i32(&buffer[i..i + 4]);
        if number == value {
            println!("FOUND {} at {:#x}", number, base_address + i);
            found.push(base_address + i);
        }
    }
    found
}

fn to_i32(slice: &[u8]) -> i32 {
    i32::from_ne_bytes(slice.try_into().unwrap())
}
