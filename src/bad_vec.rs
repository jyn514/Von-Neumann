use crate::exec_alloc::*;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use core::{fmt, ptr, slice};

/// This is basically a bad version of `Vec` that doesn't require `Vec::new_in` to be stabilized.
#[derive(PartialEq, Eq)]
pub struct ExecutableMemory {
    // NOTE: `slice.len()` is the *capacity* of the allocated memory. it may be uninitialized.
    slice: NonNull<[u8]>,
    len: usize,
}

impl ExecutableMemory {
    #[inline]
    /// Return a new region of executable memory.
    ///
    /// The region will be at least `desired_size` bytes large, but may be larger if `desired_size` is not
    /// a multiple of the page size.
    /// The memory returned will be initialized, but its contents is not specified.
    pub fn new(desired_size: usize) -> Self {
        let slice = alloc_executable_memory(desired_size).expect("failed to allocate memory");
        // SAFETY: `mmap` zero-inits memory
        ExecutableMemory {
            slice,
            len: slice.len(),
        }
    }

    /// Return a region of executable memory set to the contents of `data`.
    pub fn with_contents(data: &[u8]) -> Self {
        unsafe {
            let slice = alloc_executable_memory(data.len()).expect("failed to allocate memory");
            // SAFETY: `alloc_executable_memory` guarantees it returns a new memory allocation, so these don't overlap.
            // it also guarantees `slice` is at least `data.len()` and aligned.
            // rust's safety guarantees ensure `data.ptr()` and `data.len()` are aligned and accurate.
            ptr::copy_nonoverlapping(data.as_ptr(), slice.as_ptr().cast(), data.len());
            ExecutableMemory {
                slice,
                len: data.len(),
            }
        }
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *mut u8 {
        self.slice.as_ptr().cast()
    }
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            // SAFETY: `slice` and `len` cannot be modified outside this module, and both `new` and
            // `with_contents` guarantee that `len` bytes of `slice` are initialized.
            // this slice cannot be mutated: the only way to mutate is through `as_slice_mut`, which takes `&mut self`.
            slice::from_raw_parts(self.as_ptr(), self.len)
        }
    }
    #[inline]
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe {
            // SAFETY: `&mut self` guarantees we don't have two slices at once.
            // theoretically someone could call `unsafe { *mem.as_ptr() = x }` but that's on them to uphold the safety guarantees.
            slice::from_raw_parts_mut(self.as_ptr(), self.len)
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
            dealloc_executable_memory(self.as_ptr(), self.slice.len());
        }
    }
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

        let f: fn() -> u32 = unsafe { core::mem::transmute(memory[0..6].as_ptr()) };

        assert_eq!(f(), 4294967295);
    }

    #[test]
    #[should_panic = "don't try to allocate usize::MAX lol"]
    fn overflow() {
        ExecutableMemory::new(usize::MAX);
    }
}
