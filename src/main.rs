use clap::Parser;
use procfs::process::{Process, MMapPath};

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
    let args = Args::parse();
    println!("PID is {}", args.pid);
    let p = Process::new(args.pid).unwrap();
    // println!("{:?}", p.cwd().unwrap());
    let mut mem = p.mem().unwrap();
    let maps = p.maps().unwrap();

    for map in maps {
        match map.pathname {
            MMapPath::Heap => {
                mem.seek(SeekFrom::Start(map.address.0)).unwrap();
                let mut buf = vec![0; (map.address.1 - map.address.0) as usize];
                mem.read_exact(&mut buf).unwrap();
                for i in (0..buf.len()).step_by(4) {
                    let number = to_i32(&buf[i..i+4]);
                    if number == 424242 {
                        println!("FOUND {} at {:#x}", number, map.address.1 + i as u64);
                    }

                }
            }
            _ => ()
            
        }
        // dbg!(map.pathname);
    }
}

fn to_i32(slice: &[u8]) -> i32 {
    i32::from_ne_bytes(slice.try_into().unwrap())
}