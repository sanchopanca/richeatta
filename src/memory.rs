#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

enum State {
    Idle,
    Searching,
}

pub struct Agent {
    pid: i32,
    state: State,
    candidates: Vec<usize>,
}

impl Agent {
    pub fn new(pid: i32) -> Self {
        Agent {
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

        #[cfg(target_os = "linux")]
        linux::modify_at_address(self.pid, address, value);

        #[cfg(target_os = "windows")]
        windows::modify_at_address(self.pid, address, value);
    }

    #[cfg(target_os = "linux")]
    pub fn search(&mut self, value: i32, first_search: bool) {
        if first_search {
            self.candidates = linux::search_everywhere(self.pid, value);
        } else {
            self.candidates = linux::search_among_candidates(self.pid, value, &self.candidates);
        }
    }

    #[cfg(target_os = "windows")]
    pub fn search(&mut self, value: i32, first_search: bool) {
        if first_search {
            self.candidates = windows::search_everywhere(self.pid, value);
        } else {
            self.candidates = windows::search_among_candidates(self.pid, value, &self.candidates);
        }
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
