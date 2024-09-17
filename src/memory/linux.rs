use crate::memory::data_types::Integer;
use nix::sys::uio;
use nix::sys::uio::RemoteIoVec;
use nix::unistd::Pid;
use procfs::process::{MMapPath, Process};
use std::io::{IoSlice, Read, Seek, SeekFrom};
use std::mem::size_of;

use super::OSMemory;
use super::{first_search, to};

pub struct Linux {
    pid: i32,
}

impl Linux {
    pub fn new(pid: i32) -> Self {
        Linux { pid }
    }
}

impl<T: Integer> OSMemory<T> for Linux {
    fn modify_at_address(&self, address: usize, value: T) {
        let data = value.to_ne_bytes();
        let wrapper = IoSlice::new(&data);
        let target = RemoteIoVec {
            base: address,
            len: data.len(),
        };
        match uio::process_vm_writev(Pid::from_raw(self.pid), &[wrapper], &[target]) {
            Err(e) => println!("Error writing to {:#x}: {}", target.base, e),
            Ok(written) => println!(
                "Success writing to {:#x}, {} bytes written",
                target.base, written
            ),
        }
    }

    fn search_everywhere(&self, value: T) -> Vec<usize> {
        let p = Process::new(self.pid).unwrap();
        let mut mem = p.mem().unwrap();
        let maps = p.maps().unwrap();

        let mut candidates = Vec::new();
        for map in maps {
            if map.pathname != MMapPath::Heap {
                continue;
            }
            mem.seek(SeekFrom::Start(map.address.0)).unwrap();
            let mut buf = vec![0; (map.address.1 - map.address.0) as usize];
            mem.read_exact(&mut buf).unwrap();
            candidates.append(&mut first_search(&buf, value, map.address.0 as usize));
        }
        candidates
    }

    fn search_among_candidates(&self, value: T, candidates: &[usize]) -> Vec<usize> {
        let p = Process::new(self.pid).unwrap();
        let mut mem = p.mem().unwrap();
        let maps = p.maps().unwrap();

        let mut remaining_candidates = Vec::new();
        for map in maps {
            if map.pathname != MMapPath::Heap {
                continue;
            }
            mem.seek(SeekFrom::Start(map.address.0)).unwrap();
            let mut buf = vec![0; (map.address.1 - map.address.0) as usize];
            mem.read_exact(&mut buf).unwrap();
            remaining_candidates.append(&mut refine_search(
                &buf,
                value,
                map.address.0 as usize,
                candidates,
            ));
        }
        remaining_candidates
    }

    fn get_all_memory_regions(&self) -> Vec<super::MemoryRegion<T>> {
        unimplemented!()
    }

    fn filter_regions(
        &self,
        _regions: &[super::MemoryRegion<T>],
        _filter: fn(T, T) -> bool,
    ) -> Vec<super::MemoryRegion<T>> {
        unimplemented!()
    }
}

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
