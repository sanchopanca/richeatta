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

pub struct Windows {
    #[allow(dead_code)]
    pid: u32,
    process_handle: *mut winapi::ctypes::c_void,
}

impl Windows {
    pub fn new(pid: i32) -> Self {
        let pid = pid as u32;
        let process_handle = unsafe {
            OpenProcess(
                PROCESS_QUERY_INFORMATION
                    | PROCESS_VM_READ
                    | PROCESS_VM_WRITE
                    | PROCESS_VM_OPERATION,
                0,
                pid,
            )
        };

        if process_handle.is_null() {
            panic!(
                "Failed to open process with pid: {}. Error: {}",
                pid,
                unsafe { GetLastError() }
            );
        }

        Windows {
            pid,
            process_handle,
        }
    }
}

impl Drop for Windows {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.process_handle);
        }
    }
}

impl<T: Integer> OSMemory<T> for Windows {
    fn modify_at_address(&self, address: usize, value: T) {
        let value = value.to_ne_bytes();

        // Attempt to write data to the specified memory address
        let mut bytes_written = 0;
        let write_result = unsafe {
            WriteProcessMemory(
                self.process_handle,
                address as *mut _,
                value.as_ptr() as *const _,
                value.len(),
                &mut bytes_written,
            )
        };

        if write_result == 0 {
            eprintln!("Failed to write memory");
            #[allow(clippy::needless_return)]
            return;
        }
    }

    fn search_everywhere(&self, value: T) -> Vec<usize> {
        let mut candidates = Vec::new();
        let mut base_address = 0;
        let mut memory_info: MEMORY_BASIC_INFORMATION = unsafe { zeroed() };

        while unsafe {
            VirtualQueryEx(
                self.process_handle,
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
                        self.process_handle,
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
        candidates
    }

    fn search_among_candidates(&self, value: T, candidates: &[usize]) -> Vec<usize> {
        let mut remaining_candidates = Vec::new();

        for &address in candidates {
            let mut bytes_read = 0;
            let mut value_at_address = T::new();
            let read_success = unsafe {
                ReadProcessMemory(
                    self.process_handle,
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
