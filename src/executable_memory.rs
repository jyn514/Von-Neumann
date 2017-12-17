use core::{slice, mem, fmt};
use core::ops::{Deref, DerefMut};

#[cfg(any(target_os = "linux", target_os = "macos"))]
use libc;

#[cfg(target_os = "windows")]
use winapi;
#[cfg(target_os = "windows")]
use kernel32;


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
        ExecutableMemory {
            ptr: unsafe {
                alloc_executable_memory(PAGE_SIZE, num_pages)
            },
            len: num_pages * PAGE_SIZE,
        }
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr
    }
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
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
            dealloc_executable_memory(self.ptr, PAGE_SIZE);
        }
    }
}


#[cfg(any(target_os = "linux", target_os = "macos"))]
unsafe fn alloc_executable_memory(page_size: usize, num_pages: usize) -> *mut u8 {
    let size = page_size * num_pages;
    let mut raw_addr: *mut libc::c_void = mem::uninitialized();

    libc::posix_memalign(&mut raw_addr, page_size, size);
    libc::mprotect(raw_addr, size, libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE);

    mem::transmute(raw_addr)
}
#[cfg(target_os = "windows")]
unsafe fn alloc_executable_memory(page_size: usize, num_pages: usize) -> *mut u8 {
    let size = (page_size * num_pages) as u64;
    let raw_addr: *mut winapi::c_void;

    raw_addr = kernel32::VirtualAlloc(
        ::core::ptr::null_mut(),
        size,
        winapi::MEM_RESERVE | winapi::MEM_COMMIT,
        winapi::winnt::PAGE_EXECUTE_READWRITE
    );
    if raw_addr == 0 as *mut winapi::c_void {
        panic!("Could not allocate memory. Error Code: {:?}", kernel32::GetLastError());
    }

    let old_prot: *mut winapi::DWORD = mem::uninitialized();
    let result = kernel32::VirtualProtect(
        raw_addr,
        size,
        winapi::winnt::PAGE_EXECUTE_READWRITE,
        old_prot as *mut _,
    );
    if result == 0 {
        panic!("Could not protect allocated memory. Error Code: {:?}", kernel32::GetLastError());
    }

    mem::transmute(raw_addr)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
unsafe fn dealloc_executable_memory(ptr: *mut u8, page_size: usize) {
    libc::munmap(ptr as *mut _, page_size);
}
#[cfg(target_os = "windows")]
unsafe fn dealloc_executable_memory(ptr: *mut u8, _: usize) {
	kernel32::VirtualFree(ptr as *mut _, 0, winapi::MEM_RELEASE);
}


#[cfg(test)]
mod test {
    use super::*;


    #[test]
    fn test_call_function() {
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
