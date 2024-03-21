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
        self.write_process_memory(address, &value).unwrap();
    }

    fn search_everywhere(&self, value: T) -> Vec<usize> {
        let mut candidates = Vec::new();

        for region in self.memory_regions() {
            // Only consider regions that are committed, ignoring other states for simplicity
            if region.State != winapi::um::winnt::MEM_COMMIT {
                continue;
            }
            let mut buffer: Vec<u8> = vec![0; region.RegionSize];
            if let Err(err) = self.read_process_memory(region.BaseAddress as usize, &mut buffer) {
                eprintln!("{}", err);
            } else {
                candidates.append(&mut first_search(
                    &buffer,
                    value,
                    region.BaseAddress as usize,
                ));
            }
        }
        candidates
    }

    fn search_among_candidates(&self, value: T, candidates: &[usize]) -> Vec<usize> {
        let mut remaining_candidates = Vec::new();

        for &address in candidates {
            let values_at_address = self.read_process_memory_single_value::<T>(address).unwrap();
            if values_at_address == value {
                println!("FOUND {} at {:#x}", value, address);
                remaining_candidates.push(address);
            }
        }
        remaining_candidates
    }
}

impl Windows {
    fn write_process_memory(&self, address: usize, value: &[u8]) -> Result<(), WinError> {
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
            let error = unsafe { GetLastError() };
            return Err(error.into());
        }

        assert!(bytes_written == value.len());

        Ok(())
    }

    fn read_process_memory<T: Integer>(
        &self,
        address: usize,
        buffer: &mut [T],
    ) -> Result<(), WinError> {
        let mut bytes_read = 0;

        let len_in_bytes = std::mem::size_of_val(buffer);

        let read_success = unsafe {
            ReadProcessMemory(
                self.process_handle,
                address as *const _,
                buffer.as_mut_ptr() as *mut _,
                len_in_bytes,
                &mut bytes_read,
            )
        };

        if read_success == 0 {
            let error = unsafe { GetLastError() };
            return Err(error.into());
        }

        assert!(bytes_read == len_in_bytes);

        Ok(())
    }

    fn read_process_memory_single_value<T: Integer>(&self, address: usize) -> Result<T, WinError> {
        let mut value = [T::new()];
        self.read_process_memory(address, &mut value)?;
        Ok(value[0])
    }

    fn memory_regions(&self) -> MemoryRegionIterator {
        MemoryRegionIterator {
            process_handle: self.process_handle,
            memory_info: unsafe { zeroed() },
        }
    }
}

struct MemoryRegionIterator {
    process_handle: *mut winapi::ctypes::c_void,
    memory_info: MEMORY_BASIC_INFORMATION,
}

impl Iterator for MemoryRegionIterator {
    type Item = MEMORY_BASIC_INFORMATION;

    fn next(&mut self) -> Option<Self::Item> {
        let next_base_address =
            (self.memory_info.BaseAddress as usize) + self.memory_info.RegionSize;
        if unsafe {
            VirtualQueryEx(
                self.process_handle,
                next_base_address as *const _,
                &mut self.memory_info,
                size_of::<MEMORY_BASIC_INFORMATION>(),
            )
        } == size_of::<MEMORY_BASIC_INFORMATION>()
        {
            Some(self.memory_info)
        } else {
            None
        }
    }
}

enum WinError {
    PartialCopy,
    OtherError,
}

impl From<u32> for WinError {
    fn from(err: u32) -> Self {
        match err {
            0x12B => WinError::PartialCopy,
            _ => {
                eprintln!("Unknown error: {}", err);
                WinError::OtherError
            }
        }
    }
}

impl std::fmt::Debug for WinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for WinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            WinError::PartialCopy => "Error: Partial copy. Only part of a ReadProcessMemory or WriteProcessMemory request was completed.",
            WinError::OtherError => "Unknown error",
        };
        write!(f, "{}", message)
    }
}
