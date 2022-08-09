use nix::sys::uio;
use nix::sys::uio::RemoteIoVec;
use nix::unistd::Pid;
use procfs::process::{Process, MMapPath};

use std::io::IoSlice;
use std::io::prelude::*;
use std::io::SeekFrom;

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

        let data = value.to_ne_bytes();
        let wrapper = IoSlice::new(&data);
        let target = RemoteIoVec {
            base: address,
            len: data.len(),
        };
        match uio::process_vm_writev(Pid::from_raw(self.pid), &[wrapper], &[target]) {
            Err(e) => println!("Error writing to {:#x}: {}", target.base, e),
            Ok(written) => println!("Success writing to {:#x}, {} bytes written", target.base, written)
        }
    }

    pub fn search(&mut self, value: i32, first_search: bool) {
        let p = Process::new(self.pid).unwrap();
        let mut mem = p.mem().unwrap();
        let maps = p.maps().unwrap();

        for map in maps {
            match map.pathname {
                MMapPath::Heap => {
                    mem.seek(SeekFrom::Start(map.address.0)).unwrap();
                    let mut buf = vec![0; (map.address.1 - map.address.0) as usize];
                    mem.read_exact(&mut buf).unwrap();
                    if first_search {
                        self.first_search_helper(&buf, value, map.address.0 as usize);
                    } else {
                        self.refine_search_helper(&buf, value, map.address.0 as usize);
                    }
                },
                _ => ()
            };
        }
    }

    fn first_search_helper(&mut self, buffer: &[u8], value: i32, address_start: usize) {
        let mut found = Vec::new();
        for i in (0..buffer.len()).step_by(4) {
            let number = to_i32(&buffer[i..i+4]);
            if number == value {
                println!("FOUND {} at {:#x}", number, address_start + i);
                found.push(address_start + i);
            }
        }
        self.candidates = found;
    }

    fn refine_search_helper(&mut self, buffer: &[u8], value: i32, address_start: usize) {
        let mut found = Vec::new();
        for address in &self.candidates {
            let i = address - address_start;
            let number = to_i32(&buffer[i..i+4]);
            if number == value {
                println!("FOUND {} at {:#x}", number, address_start + i);
                found.push(address_start + i);
            }
        }
        self.candidates = found;
    }
}

fn to_i32(slice: &[u8]) -> i32 {
    i32::from_ne_bytes(slice.try_into().unwrap())
}
