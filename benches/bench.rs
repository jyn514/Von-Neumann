#![feature(test)]

extern crate test;
use test::Bencher;
use vonneumann::ExecutableMemory;

use std::mem;


#[bench]
fn bench_native(b: &mut Bencher) {

    #[inline(never)]
    fn f() -> u32 { test::black_box(4294967295) }

    b.iter(f);
}

#[bench]
fn bench_executable_memory(b: &mut Bencher) {
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

    b.iter(f);
}
