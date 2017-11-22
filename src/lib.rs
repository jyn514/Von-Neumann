#![no_std]


extern crate libc;
#[cfg(target_family = "windows")] extern crate kernel32;


mod executable_memory;


pub use self::executable_memory::ExecutableMemory;
