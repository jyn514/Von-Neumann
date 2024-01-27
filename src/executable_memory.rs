use core::{slice, mem, fmt};
use core::ops::{Deref, DerefMut};
use core::mem::MaybeUninit;

#[derive(PartialEq, Eq)]
pub struct ExecutableMemory {
    ptr: *mut u8,
    len: usize,
}

impl ExecutableMemory {
    #[inline]
    /// Return a new region of executable memory.
    ///
    /// The region will be at least `desired_size` large, but may be larger if `desired_size` is not
    /// a multiple of the page size.
    pub fn new(desired_size: usize) -> Self {
        let (ptr, len) = unsafe { alloc_executable_memory(desired_size) };
        ExecutableMemory { ptr, len }
    }

    /// Return a region of executable memory set to the contents of `data`.
    ///
    /// The region will be rounded up to the nearest page (see [`PAGE_SIZE`]).
    /// The contents of the memory after `data.len()` is not specified.
    pub fn with_contents(data: &[u8]) -> Self {
        let mut mem = Self::new(data.len());
        mem.as_slice_mut()[..data.len()].copy_from_slice(&data);
        mem
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

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
impl DerefMut for ExecutableMemory {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

impl fmt::Debug for ExecutableMemory {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_slice().fmt(f)
    }
}

impl Drop for ExecutableMemory {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            dealloc_executable_memory(self.ptr);
        }
    }
}

/// Round `desired` up to the nearest multiple of `page_size`.
fn round_to(desired: usize, page_size: usize) -> usize {
    let rem = desired % page_size;
    if rem == 0 { desired } else { desired.checked_add(page_size - rem).expect("don't try to allocate usize::MAX lol") }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
unsafe fn alloc_executable_memory(desired: usize) -> (*mut u8, usize) {
    let page_size = libc::sysconf(libc::_SC_PAGESIZE);
    let actual = round_to(desired, page_size);

    let mut raw_addr = MaybeUninit::<*mut libc::c_void>::uninit();
    libc::posix_memalign(raw_addr.as_mut_ptr(), page_size, actual);
    let raw_addr = raw_addr.assume_init();
    libc::mprotect(raw_addr, actual, libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE);

    (mem::transmute(raw_addr), actual)
}
#[cfg(target_os = "windows")]
unsafe fn alloc_executable_memory(desired: usize) -> (*mut u8, usize) {
    use winapi::um::sysinfoapi;
    let mut sysinfo = MaybeUninit::<sysinfoapi::SYSTEM_INFO>::uninit();
    sysinfoapi::GetSystemInfo(sysinfo.as_mut_ptr());
    let page_size = sysinfo.assume_init().dwAllocationGranularity;

    let actual = round_to(desired, page_size as usize);
    let raw_addr: *mut winapi::ctypes::c_void = winapi::um::memoryapi::VirtualAlloc(
        ::core::ptr::null_mut(),
        actual,
        winapi::um::winnt::MEM_RESERVE | winapi::um::winnt::MEM_COMMIT,
        winapi::um::winnt::PAGE_EXECUTE_READWRITE
    );

    assert_ne!(
        raw_addr, 0 as *mut winapi::ctypes::c_void,
        "Could not allocate memory. Error Code: {:?}",
        winapi::um::errhandlingapi::GetLastError()
    );

    (mem::transmute(raw_addr), actual)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
unsafe fn dealloc_executable_memory(ptr: *mut u8) {
    libc::free(ptr as *mut _);
}
#[cfg(target_os = "windows")]
unsafe fn dealloc_executable_memory(ptr: *mut u8) {
	winapi::um::memoryapi::VirtualFree(ptr as *mut _, 0, winapi::um::winnt::MEM_RELEASE);
}


#[cfg(test)]
mod test {
    use super::*;


    #[test]
    fn test_call_function() {
        let mut memory = ExecutableMemory::new(1);

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

    #[test]
    #[should_panic = "don't try to allocate usize::MAX lol"]
    fn overflow() {
        ExecutableMemory::new(usize::MAX);
    }
}
