use std::mem::{size_of, zeroed};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::CloseHandle;
use winapi::um::memoryapi::{ReadProcessMemory, VirtualQueryEx, WriteProcessMemory};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winnt::{
    MEMORY_BASIC_INFORMATION, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ,
    PROCESS_VM_WRITE,
};

use super::data_types::Integer;
use super::first_search;
use super::OSMemory;

pub struct Windows;

impl Windows {
    pub fn new() -> Self {
        Windows
    }
}

impl <T: Integer> OSMemory<T> for Windows {
    fn modify_at_address(&self, pid: i32, address: usize, value: T) {
        let value = value.to_ne_bytes();

        // Open a handle to the process with write and operation permissions
        let process_handle =
            unsafe { OpenProcess(PROCESS_VM_WRITE | PROCESS_VM_OPERATION, 0, pid as u32) };

        if process_handle.is_null() {
            eprintln!("Failed to open process");
            return;
        }

        // Attempt to write data to the specified memory address
        let mut bytes_written = 0;
        let write_result = unsafe {
            WriteProcessMemory(
                process_handle,
                address as *mut _,
                value.as_ptr() as *const _,
                value.len(),
                &mut bytes_written,
            )
        };

        // Close the handle after the operation
        unsafe {
            CloseHandle(process_handle);
        }

        if write_result == 0 {
            eprintln!("Failed to write memory");
            #[allow(clippy::needless_return)]
            return;
        }
    }

    fn search_everywhere(&self, pid: i32, value: T) -> Vec<usize> {
        let process_handle =
            unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid as u32) };

        if process_handle.is_null() {
            println!("Failed to open process. Error: {}", unsafe {
                GetLastError()
            });
            return Vec::new();
        }

        let mut candidates = Vec::new();
        let mut base_address = 0;
        let mut memory_info: MEMORY_BASIC_INFORMATION = unsafe { zeroed() };

        while unsafe {
            VirtualQueryEx(
                process_handle,
                base_address as *const _,
                &mut memory_info,
                size_of::<MEMORY_BASIC_INFORMATION>(),
            )
        } == size_of::<MEMORY_BASIC_INFORMATION>()
        {
            // Only consider regions that are committed, ignoring other states for simplicity
            if memory_info.State == winapi::um::winnt::MEM_COMMIT {
                let mut buffer: Vec<u8> = vec![0; memory_info.RegionSize];
                let mut bytes_read = 0;

                let read_success = unsafe {
                    ReadProcessMemory(
                        process_handle,
                        memory_info.BaseAddress,
                        buffer.as_mut_ptr() as *mut _,
                        memory_info.RegionSize,
                        &mut bytes_read,
                    )
                };

                if read_success != 0 && bytes_read > 0 {
                    candidates.append(&mut first_search(&buffer, value, base_address));
                }
            }

            // Move to the next memory region
            base_address = (memory_info.BaseAddress as usize) + memory_info.RegionSize;
        }
        unsafe {
            CloseHandle(process_handle);
        }
        candidates
    }

    fn search_among_candidates(&self, pid: i32, value: T, candidates: &[usize]) -> Vec<usize> {
        let process_handle =
            unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid as u32) };

        if process_handle.is_null() {
            println!("Failed to open process. Error: {}", unsafe {
                GetLastError()
            });
            return Vec::new();
        }

        let mut remaining_candidates = Vec::new();

        for &address in candidates {
            let mut bytes_read = 0;
            let mut value_at_address = T::new();
            let read_success = unsafe {
                ReadProcessMemory(
                    process_handle,
                    address as *const _,
                    &mut value_at_address as *mut _ as *mut _,
                    size_of::<T>(),
                    &mut bytes_read,
                )
            };
            if read_success != 0 && bytes_read > 0 && value_at_address == value {
                println!("FOUND {} at {:#x}", value, address);
                remaining_candidates.push(address);
            }
        }
        remaining_candidates
    }
}