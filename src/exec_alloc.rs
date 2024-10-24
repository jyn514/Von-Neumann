use core::ptr;
use core::ptr::NonNull;

/// Round `desired` up to the nearest multiple of `page_size`.
fn round_to(desired: usize, page_size: usize) -> usize {
    let rem = desired % page_size;
    if rem == 0 {
        desired
    } else {
        desired
            .checked_add(page_size - rem)
            .expect("don't try to allocate usize::MAX lol")
    }
}

pub(crate) fn alloc_executable_memory(desired: usize) -> Result<NonNull<[u8]>, ()> {
    // https://doc.rust-lang.org/std/alloc/struct.Layout.html
    assert!(
        desired <= isize::MAX as usize,
        "alloc {desired} is too big; allocating more than isize::MAX is not allowed"
    );
    impl_::alloc_executable_memory(desired)
}

pub(crate) use impl_::*;

#[cfg(any(target_os = "linux", target_os = "macos"))]
use unix as impl_;
#[cfg(any(target_os = "linux", target_os = "macos"))]
mod unix {
    use super::*;

    pub(crate) fn page_size() -> usize {
        // SAFETY: sysconf has no validity requirements
        let size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
        assert_ne!(size, -1, "sysconf error");
        size as usize
    }

    pub(super) fn alloc_executable_memory(desired: usize) -> Result<NonNull<[u8]>, ()> {
        let actual = round_to(desired, page_size());
        unsafe {
            let ptr = libc::mmap(
                ptr::null_mut(),
                actual,
                libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            );
            if ptr == libc::MAP_FAILED {
                Err(())
            } else {
                match NonNull::new(ptr.cast()) {
                    Some(ptr) => Ok(NonNull::slice_from_raw_parts(ptr, actual)),
                    // NOTE: it's actually valid for mmap to point to the 0 address. but rust's allocator design rules this out.
                    // oops!
                    None => Err(()),
                }
            }
        }
    }

    /// SAFETY: `ptr` must have been allocated by `allocate_executable_memory` and point to `cap` bytes of memory
    pub(crate) unsafe fn dealloc_executable_memory(ptr: *mut u8, cap: usize) {
        libc::munmap(ptr as *mut _, cap);
    }
}

#[cfg(target_os = "windows")]
use windows as impl_;
#[cfg(target_os = "windows")]
mod windows {
    use super::*;

    pub(crate) fn page_size() -> usize {
        use core::mem::MaybeUninit;
        use winapi::um::sysinfoapi;

        let mut sysinfo = MaybeUninit::<sysinfoapi::SYSTEM_INFO>::uninit();
        unsafe {
            // SAFETY: GetSystemInfo has no validity requirements
            sysinfoapi::GetSystemInfo(sysinfo.as_mut_ptr());
            // SAFETY: GetSystemInfo initializes `sysinfo` and cannot error
            sysinfo.assume_init().dwAllocationGranularity as usize
        }
    }

    pub(crate) fn alloc_executable_memory(desired: usize) -> Result<NonNull<[u8]>, ()> {
        let actual = round_to(desired, page_size());
        let raw_addr = unsafe {
            winapi::um::memoryapi::VirtualAlloc(
                ptr::null_mut(),
                actual,
                winapi::um::winnt::MEM_RESERVE | winapi::um::winnt::MEM_COMMIT,
                winapi::um::winnt::PAGE_EXECUTE_READWRITE,
            )
        };

        match NonNull::new(raw_addr.cast()) {
            Some(ptr) => Ok(NonNull::slice_from_raw_parts(ptr, actual)),
            None => Err(()),
        }
    }

    /// SAFETY: `ptr` must be non-null and come from an allocation returned by `alloc_executable_memory`.`
    pub(crate) unsafe fn dealloc_executable_memory(ptr: *mut u8, _: usize) {
        winapi::um::memoryapi::VirtualFree(ptr as *mut _, 0, winapi::um::winnt::MEM_RELEASE);
    }
}
