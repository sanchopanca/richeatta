use caps;
use clap::Parser;
use nix::sys::uio;
use nix::sys::uio::RemoteIoVec;
use nix::unistd::Pid;
use procfs::process::{Process, MMapPath};

use std::io::IoSlice;
use std::io::prelude::*;
use std::io::SeekFrom;


#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
   /// PID of the process to inspect
   #[clap(short, long, value_parser)]
   pid: i32,
}

fn main() {
    println!("{}", caps::has_cap(None, caps::CapSet::Permitted, caps::Capability::CAP_SYS_PTRACE).unwrap());
    let args = Args::parse();
    println!("PID is {}", args.pid);
    let p = Process::new(args.pid).unwrap();
    // println!("{:?}", p.cwd().unwrap());
    let mut mem = p.mem().unwrap();
    let maps = p.maps().unwrap();

    for map in maps {
        let address = match map.pathname {
            MMapPath::Heap => {
                let mut address = None;
                mem.seek(SeekFrom::Start(map.address.0)).unwrap();
                let mut buf = vec![0; (map.address.1 - map.address.0) as usize];
                mem.read_exact(&mut buf).unwrap();
                for i in (0..buf.len()).step_by(4) {
                    let number = to_i32(&buf[i..i+4]);
                    if number == 424242 {
                        println!("FOUND {} at {:#x}", number, map.address.0 as usize + i);
                        address = Some(map.address.0 as usize + i);
                        // break;
                    }
                }
                address
            }
            _ => None
        };
        match address {
            Some(address) => {
                update(address, args.pid)
            }
            None => ()
        }
        
        // dbg!(map.pathname);
    }
}

fn to_i32(slice: &[u8]) -> i32 {
    i32::from_ne_bytes(slice.try_into().unwrap())
}

fn update(address: usize, pid: i32) {
    let data = 999999_i32.to_ne_bytes();
    let wrapper = IoSlice::new(&data);
    let target = RemoteIoVec {
        base: address,
        len: data.len(),
    };
    match uio::process_vm_writev(Pid::from_raw(pid), &[wrapper], &[target]) {
        Err(e) => println!("Error writing to {:#x}: {}", target.base, e),
        Ok(written) => println!("Success writing to {:#x}, {} bytes written", target.base, written)
    }
}
