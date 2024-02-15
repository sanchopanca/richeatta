use nix::sys::uio;
use nix::sys::uio::RemoteIoVec;
use nix::unistd::Pid;
use procfs::process::{MMapPath, Process};
use std::io::{IoSlice, Read, Seek, SeekFrom};

use super::first_search;
use super::refine_search;

pub fn modify_at_address(pid: i32, address: usize, value: i32) {
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

pub fn search_everywhere(pid: i32, value: i32) -> Vec<usize> {
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

pub fn search_among_candidates(pid: i32, value: i32, candidates: &[usize]) -> Vec<usize> {
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
