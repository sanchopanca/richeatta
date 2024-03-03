use nix::sys::uio;
use nix::sys::uio::RemoteIoVec;
use nix::unistd::Pid;
use procfs::process::{MMapPath, Process};
use std::io::{IoSlice, SeekFrom};
use std::mem::size_of;
use crate::memory::data_types::Integer;

use super::{first_search, to};
use super::OSMemory;

pub struct Linux;

impl Linux {
    pub fn new() -> Self {
        Linux
    }
}

impl<T: Integer> OSMemory<T> for Linux {
    fn modify_at_address(&self, pid: i32, address: usize, value: T) {
        let data = value.to_ne_bytes();
        let wrapper = IoSlice::new(&data);
        let target = RemoteIoVec {
            base: address,
            len: data.len(),
        };
        match uio::process_vm_writev(Pid::from_raw(pid), &[wrapper], &[target]) {
            Err(e) => println!("Error writing to {:#x}: {}", target.base, e),
            Ok(written) => println!(
                "Success writing to {:#x}, {} bytes written",
                target.base, written
            ),
        }
    }

    fn search_everywhere(&self, pid: i32, value: T) -> Vec<usize> {
        let p = Process::new(pid).unwrap();
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

    fn search_among_candidates(&self, pid: i32, value: T, candidates: &[usize]) -> Vec<usize> {
        let p = Process::new(pid).unwrap();
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
            candidates.append(&mut refine_search(
                &buf,
                value,
                map.address.0 as usize,
                &candidates,
            ));
        }
        candidates
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