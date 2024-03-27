use std::mem::size_of;

use self::data_types::Integer;

mod data_types;

trait OSMemory<T: Integer> {
    fn modify_at_address(&self, address: usize, value: T);
    fn search_everywhere(&self, value: T) -> Vec<usize>;
    fn search_among_candidates(&self, value: T, candidates: &[usize]) -> Vec<usize>;
    fn get_all_memory_regions(&self) -> Vec<MemoryRegion<T>>;
    fn filter_regions(
        &self,
        regions: &[MemoryRegion<T>],
        filter: fn(T, T) -> bool,
    ) -> Vec<MemoryRegion<T>>;
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

struct MemoryRegion<T: Integer> {
    base_address: usize,
    data: Vec<T>,
}

pub struct Process {
    pid: i32,
}

impl Process {
    pub fn new(pid: i32) -> Self {
        Process { pid }
    }

    pub fn search_known_value<T: Integer>(&self, value: T) -> KnownValueSearch<T> {
        let os = Box::new(CurrentOS::new(self.pid));
        let candidates = os.search_everywhere(value);
        KnownValueSearch::new(candidates, os)
    }

    pub fn search_unknown_value<T: Integer>(&self) -> UnknownValueSearch<T> {
        let os = Box::new(CurrentOS::new(self.pid));
        let regions = os.get_all_memory_regions();
        UnknownValueSearch { os, regions }
    }
}

pub struct KnownValueSearch<T: Integer> {
    candidates: Vec<usize>,
    os: Box<dyn OSMemory<T>>,
}

pub struct UnknownValueSearch<T: Integer> {
    os: Box<dyn OSMemory<T>>,
    regions: Vec<MemoryRegion<T>>,
}

impl<T: Integer> KnownValueSearch<T> {
    fn new(candidates: Vec<usize>, os: Box<dyn OSMemory<T>>) -> Self {
        KnownValueSearch { candidates, os }
    }

    pub fn count(&self) -> usize {
        self.candidates.len()
    }

    pub fn refine(&mut self, new_value: T) {
        self.candidates = self.os.search_among_candidates(new_value, &self.candidates);
    }

    pub fn modify(&self, value: T) {
        let address = self.candidates[0];
        self.os.modify_at_address(address, value);
    }
}

impl<T: Integer> UnknownValueSearch<T> {
    pub fn count(&self) -> usize {
        self.regions.iter().map(|region| region.data.len()).sum()
    }

    pub fn value_increased(&mut self) {
        self.refine(|old_value: T, new_value: T| new_value > old_value);
    }

    pub fn value_decreased(&mut self) {
        self.refine(|old_value: T, new_value: T| new_value < old_value);
    }

    pub fn value_didnt_change(&mut self) {
        self.refine(|old_value: T, new_value: T| new_value == old_value);
    }

    pub fn value_changed(&mut self) {
        self.refine(|old_value: T, new_value: T| new_value != old_value);
    }

    pub fn modify(&self, value: T) {
        self.os
            .modify_at_address(self.regions[0].base_address, value)
    }

    pub fn get_current_value(&self) -> T {
        self.regions[0].data[0]
    }

    fn refine(&mut self, filter: fn(T, T) -> bool) {
        self.regions = self.os.filter_regions(&self.regions, filter);
    }
}

fn first_search<T: Integer>(buffer: &[u8], value: T, base_address: usize) -> Vec<usize> {
    let size = size_of::<T>();
    let mut found = Vec::new();
    for i in (0..buffer.len()).step_by(size) {
        let number = to::<T>(&buffer[i..i + size]);
        if number == value {
            // println!("FOUND {} at {:#x}", number, base_address + i);
            found.push(base_address + i);
        }
    }
    found
}

fn to<T: Integer>(slice: &[u8]) -> T {
    T::from_ne_bytes(slice)
}
