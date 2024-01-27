#![no_std]


#[cfg(any(target_os = "linux", target_os = "macos"))]
extern crate libc;
#[cfg(target_os = "windows")]
extern crate winapi;


mod executable_memory;


pub use self::executable_memory::{PAGE_SIZE, ExecutableMemory};
