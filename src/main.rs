#![no_std]
#![no_main]

mod allocator;

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;

use allocator::SimpleAllocator;
use core::panic::PanicInfo;

#[global_allocator]
static GLOBAL_ALLOCATOR: SimpleAllocator = SimpleAllocator;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // 测试 1: 基本分配和释放
    let boxed = Box::new(42u32);
    let value = *boxed;
    drop(boxed);

    // 测试 2: Vec 分配
    let mut vec = Vec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);

    // 测试 3: 重新分配
    let mut vec2 = Vec::with_capacity(4);
    for i in 0..10 {
        vec2.push(i);
    }

    // 输出测试完成
    let msg = b"Allocator tests passed!\n";
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") 1usize,
            in("rdi") 1usize,
            in("rsi") msg.as_ptr(),
            in("rdx") msg.len(),
        );
    }

    let msg = b"Hello, RCore!\n";
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") 1usize, // sys_write
            in("rdi") 1usize, // stdout
            in("rsi") msg.as_ptr(),
            in("rdx") msg.len(),
        );
        core::arch::asm!(
            "syscall",
            in("rax") 60usize, // sys_exit
            in("rdi") 0usize, // exit code
        );
    }
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
