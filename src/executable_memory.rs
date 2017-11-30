use core::{slice, mem, fmt};
use core::ops::{Deref, DerefMut};

use libc;
#[cfg(target_family = "windows")] use kernel32;


pub const PAGE_SIZE: usize = 4096;


#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExecutableMemory {
    ptr: *mut u8,
    len: usize,
}

impl Default for ExecutableMemory {
    #[inline(always)]
    fn default() -> Self {
        ExecutableMemory::new(1)
    }
}

impl ExecutableMemory {
    #[inline]
    pub fn new(num_pages: usize) -> Self {
        let len = num_pages * PAGE_SIZE;

        ExecutableMemory {
            ptr: unsafe {
                mem::transmute(alloc_executable_memory(PAGE_SIZE, len))
            },
            len: len,
        }
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr
    }
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self.ptr, self.len)
        }
    }
    #[inline]
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(self.ptr, self.len)
        }
    }
}

impl Deref for ExecutableMemory {
    type Target = [u8];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
impl DerefMut for ExecutableMemory {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

impl fmt::Debug for ExecutableMemory {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_slice().fmt(f)
    }
}

impl Drop for ExecutableMemory {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            dealloc_executable_memory(mem::transmute(self.ptr), PAGE_SIZE);
        }
    }
}


#[cfg(target_family = "unix")]
unsafe fn alloc_executable_memory(page_size: usize, len: usize) -> *mut libc::c_void {
    let mut ptr: *mut libc::c_void = mem::uninitialized();

    libc::posix_memalign(&mut ptr, page_size, len);
    libc::mprotect(ptr, len, libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE);

    ptr
}
#[cfg(target_family = "windows")]
unsafe fn alloc_executable_memory(page_size: usize, len: usize) -> *mut libc::c_void {
    let mut ptr: *mut libc::c_void = mem::uninitialized();

    kernel32::VirtualAlloc(
        ptr,
        page_size,
        kernel32::MEM_RESERVE | kernel32::MEM_COMMIT,
        kernel32::PAGE_READWRITE
    );
    kernel32::VirtualProtect(
        ptr,
        len,
        kernel32::PAGE_EXECUTE_READ,
        0
    );

    ptr
}

#[cfg(target_family = "unix")]
unsafe fn dealloc_executable_memory(ptr: *mut libc::c_void, page_size: usize) {
    libc::munmap(ptr, page_size);
}
#[cfg(target_family = "windows")]
unsafe fn dealloc_executable_memory(ptr: *mut libc::c_void, page_size: usize) {
    kernel32::VirtualFree(ptr, 0, kernel32::MEM_RELEASE);
}


#[cfg(test)]
mod test {
    use super::*;


    #[test]
    fn call_function() {
        let mut memory = ExecutableMemory::default();

        memory[0] = 0xb8;
        memory[1] = 0xff;
        memory[2] = 0xff;
        memory[3] = 0xff;
        memory[4] = 0xff;
        memory[5] = 0xc3;

        let f: fn() -> u32 = unsafe {
            mem::transmute((&memory[0..6]).as_ptr())
        };

        assert_eq!(f(), 4294967295);
    }
}
