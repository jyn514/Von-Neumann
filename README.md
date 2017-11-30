rs-executable_memory
=====

executable memory for windows and unix

```rust
extern crate executable_memory;


use executable_memory::ExecutableMemory;


fn main() {
    let mut memory = ExecutableMemory::default(); // Page size 1

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
```
