use std::mem::size_of;

use self::data_types::Integer;

mod data_types;

pub trait OSMemory<T: Integer> {
    fn modify_at_address(&self, pid: i32, address: usize, value: T);
    fn search_everywhere(&self, pid: i32, value: T) -> Vec<usize>;
    fn search_among_candidates(&self, pid: i32, value: T, candidates: &[usize]) -> Vec<usize>;
}

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux::Linux as CurrentOS;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows::Windows as CurrentOS;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
use macos::MacOS as CurrentOS;

pub struct Process<T: Integer> {
    pid: i32,
    candidates: Vec<usize>,
    os: Box<dyn OSMemory<T>>,
}

impl<T: Integer> Process<T> {
    pub fn new(pid: i32) -> Self {
        Process {
            pid,
            candidates: Vec::new(),
            os: Box::new(CurrentOS::new()),
        }
    }

    pub fn count(&self) -> usize {
        self.candidates.len()
    }

    pub fn modify(&self, value: T) {
        let address = self.candidates[0];
        self.os.modify_at_address(self.pid, address, value);
    }

    pub fn search(&mut self, value: T) {
        self.candidates = self.os.search_everywhere(self.pid, value);
    }

    pub fn refine(&mut self, new_value: T) {
        self.candidates = self
            .os
            .search_among_candidates(self.pid, new_value, &self.candidates);
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

fn to<T: Integer>(slice: &[u8]) -> T {
    T::from_ne_bytes(slice)
}
