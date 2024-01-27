#![doc = include_str!("../README.md")]
#![no_std]
#![cfg_attr(feature = "nightly", feature(allocator_api, doc_auto_cfg))]
//! # Soundness
//!
//! The Rust Abstract Machine is clueless about the fact that
//! memory can be executable so i'm off the hook

mod exec_alloc;

mod bad_vec;
pub use self::bad_vec::ExecutableMemory;

#[cfg(feature = "nightly")]
pub use alloc_api::{ExecutableAllocator, Vec};
#[cfg(feature = "nightly")]
mod alloc_api {
    extern crate alloc;

    use crate::exec_alloc;
    use core::alloc::AllocError;

    /// A [`Vec`](alloc::vec::Vec) backed by RWX memory.
    ///
    /// See [`ExecutableAllocator`].
    pub type Vec<T> = alloc::vec::Vec<T, ExecutableAllocator>;
    /// An allocator that maps pages as RWX.
    ///
    /// # example
    /// ```
    /// #![feature(allocator_api)]
    /// let mut code = Vec::with_capacity_in(2, vonneumann::ExecutableAllocator);
    /// code.push(/* idk what x86 looks like lol */ 0x90_u8);
    /// code.push(0xc3);
    /// unsafe {
    ///     let f = core::mem::transmute::<*mut u8, unsafe fn()>(code.as_mut_ptr());
    ///     f();
    /// }
    /// ```
    pub struct ExecutableAllocator;
    unsafe impl core::alloc::Allocator for ExecutableAllocator {
        fn allocate(
            &self,
            layout: core::alloc::Layout,
        ) -> Result<core::ptr::NonNull<[u8]>, core::alloc::AllocError> {
            assert!(layout.align() <= exec_alloc::page_size());
            exec_alloc::alloc_executable_memory(layout.size()).or(Err(AllocError))
        }

        unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
            exec_alloc::dealloc_executable_memory(ptr.as_ptr(), layout.size());
        }
    }
}
