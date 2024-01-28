Von Neumann
=====

executable memory for windows and unix ("von Neumann architecture")

## example

```rust
use vonneumann::ExecutableMemory;

fn main() {
    let mut memory = ExecutableMemory::new(6);

    // https://www.felixcloutier.com/x86/
    memory[0] = 0xb8;
    memory[1] = 0xff;
    memory[2] = 0xff;
    memory[3] = 0xff;
    memory[4] = 0xff;
    memory[5] = 0xc3;

    let f: fn() -> u32 = unsafe {
        std::mem::transmute((&memory[0..6]).as_ptr())
    };

    assert_eq!(f(), 4294967295);
}
```

## comparison with other executable-memory crates

### [`jit-allocator`]

this crate maps pages as RWX. this is somewhat insecure and makes it easier to escalate security vulns. [`jit-allocator`] "dual maps" pages as RX and RW, avoiding this issue.

this crate always allocates full pages, which is inefficient for small allocations, and slow if you allocate frequently. `jit-allocator` can return byte-level allocations instead.

this crate supports the [`Allocator`] API when `--features nightly` is enabled. as a result, it can be used with `Vec`, and using it requires less unsafe (basically just the transmute to turn the data into a function pointer). jit-allocator inherently cannot do this because it uses dual-mapping.

[`Allocator`]: https://doc.rust-lang.org/nightly/std/alloc/trait.Allocator.html
[`jit-allocator`]: https://docs.rs/crate/jit-allocator/

### [`memfd-exec`]

[`memfd-exec`] serves a different purpose altogether: it executes whole processes. this crate is for executing individual functions that share your process state.

[`memfd-exec`]: https://lib.rs/crates/memfd-exec

### "just handrolling it with mmap lol"

this crate automatically unmaps the memory for you on drop, and supports windows as well as unix.
unlike `memmap2` you can have the memory be executable and writable at the same time :ferrisClueless:

## faq

### is anyone actually using this?

yes! <https://github.com/chellipse/bf-jit>

### does this work on macOS?

yes!

### even with runtime hardening?

no???? the man page literally says "MAP_JIT regions are never writeable and executable simultaneously" lol

if someone wants this i can imagine an api that forces you to call `make_mut` and `make_exec` separately though. i am not doing dual-mapping. no.

### does it work on aarch64?

probably but i haven't tested 🤪

### can i use this in prod?

lol please do i want to watch your code burn

as a reminder THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED
