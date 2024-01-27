Von Neumann
=====

executable memory for windows and unix ("von Neumann architecture")

```rust
use vonneumann::ExecutableMemory;

fn main() {
    let mut memory = ExecutableMemory::default(); // Page size 1

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
