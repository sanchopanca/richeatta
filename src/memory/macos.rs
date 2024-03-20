use std::ffi::c_int;
use std::mem::size_of;

use mach::kern_return::{KERN_FAILURE, KERN_SUCCESS};
use mach::message::mach_msg_type_number_t;
use mach::port::mach_port_name_t;
use mach::traps::task_for_pid;
use mach::vm::mach_vm_region;
use mach::vm::{mach_vm_read_overwrite, mach_vm_write};
use mach::vm_region::{vm_region_basic_info_data_64_t, VM_REGION_BASIC_INFO_64};
use mach::vm_types::{mach_vm_address_t, mach_vm_size_t, vm_offset_t};

use crate::memory::data_types::Integer;

use super::{first_search, OSMemory};

pub struct MacOS {
    pid: c_int,
}

impl MacOS {
    pub fn new(pid: i32) -> Self {
        MacOS { pid: pid as c_int }
    }
}

impl<T: Integer> OSMemory<T> for MacOS {
    fn modify_at_address(&self, address: usize, value: T) {
        let mut task = 0;
        let result = unsafe { task_for_pid(mach::traps::mach_task_self(), self.pid, &mut task) };
        if result != KERN_SUCCESS {
            eprintln!(
                "Failed to get task for pid: {}. Error code: {}",
                self.pid, result
            );
            if result == KERN_FAILURE {
                eprintln!("Error is likely due to System Integrity Protection enabled");
                eprintln!("See https://support.apple.com/en-us/102149 for details");
                eprintln!(
                    "If it's disabled (check with `csrutil status`), run the program with sudo"
                );
            }
            return;
        }

        let data = value.to_ne_bytes();
        let result = unsafe {
            mach_vm_write(
                task,
                address as mach_vm_address_t,
                data.as_ptr() as vm_offset_t,
                data.len() as mach_msg_type_number_t,
            )
        };

        if result != KERN_SUCCESS {
            eprintln!("Failed to write to memory. Error code: {}", result);
        }
    }

    fn search_everywhere(&self, value: T) -> Vec<usize> {
        let mut found = Vec::new();
        let mut task = 0;
        let result = unsafe { task_for_pid(mach::traps::mach_task_self(), self.pid, &mut task) };
        if result != KERN_SUCCESS {
            eprintln!(
                "Failed to get task for pid: {}. Error code: {}",
                self.pid, result
            );
            if result == KERN_FAILURE {
                eprintln!("Error is likely due to System Integrity Protection enabled");
                eprintln!("See https://support.apple.com/en-us/102149 for details");
                eprintln!(
                    "If it's disabled (check with `csrutil status`), run the program with sudo"
                );
            }
            return found;
        }

        let mut address = 0;
        let mut size = 0;
        let mut info_count = (size_of::<vm_region_basic_info_data_64_t>() / size_of::<T>())
            as mach_msg_type_number_t;
        let mut object_name: mach_port_name_t = 0;
        let mut info = vm_region_basic_info_data_64_t::default();

        loop {
            let result = unsafe {
                mach_vm_region(
                    task,
                    &mut address,
                    &mut size,
                    VM_REGION_BASIC_INFO_64,
                    &mut info as *mut _ as *mut _,
                    &mut info_count,
                    &mut object_name,
                )
            };

            if result != KERN_SUCCESS {
                break;
            }

            let chunk_size = 100 * 1024 * 1024; // 100MB
            let mut offset: usize = 0;

            while offset < size as usize {
                let read_size = if size as usize - offset > chunk_size {
                    chunk_size
                } else {
                    size as usize - offset
                };

                let mut buffer = vec![0; read_size];
                let mut out_size: mach_vm_size_t = 0;

                let result = unsafe {
                    mach_vm_read_overwrite(
                        task,
                        address + offset as u64,
                        read_size as mach_vm_size_t,
                        buffer.as_mut_ptr() as mach_vm_address_t,
                        &mut out_size,
                    )
                };

                if result == KERN_SUCCESS {
                    let numbers = first_search(&buffer, value, address as usize + offset);
                    found.extend(numbers);
                }

                offset += read_size;
            }

            address += size;
        }

        found
    }

    fn search_among_candidates(&self, value: T, candidates: &[usize]) -> Vec<usize> {
        let mut found = Vec::new();
        let mut task = 0;
        let result = unsafe { task_for_pid(mach::traps::mach_task_self(), self.pid, &mut task) };
        if result != KERN_SUCCESS {
            eprintln!(
                "Failed to get task for pid: {}. Error code: {}",
                self.pid, result
            );
            if result == KERN_FAILURE {
                eprintln!("Error is likely due to System Integrity Protection enabled");
                eprintln!("See https://support.apple.com/en-us/102149 for details");
                eprintln!(
                    "If it's disabled (check with `csrutil status`), run the program with sudo"
                );
            }
            return found;
        }

        for &address in candidates {
            let mut buffer = vec![0; size_of::<T>()];
            let mut out_size: mach_vm_size_t = 0;

            let result = unsafe {
                mach_vm_read_overwrite(
                    task,
                    address as mach_vm_address_t,
                    buffer.len() as mach_vm_size_t,
                    buffer.as_mut_ptr() as mach_vm_address_t,
                    &mut out_size,
                )
            };

            if result == KERN_SUCCESS {
                let read_value = T::from_ne_bytes(&buffer);
                if read_value == value {
                    found.push(address);
                }
            }
        }

        found
    }
}
