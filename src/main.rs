#![no_std]
#![no_main]

#[global_allocator]
static GLOBAL_ALLOCATOR: SimpleAllocator = SimpleAllocator::new();

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;
use new_rcore::allocator::SimpleAllocator;
use new_rcore::{sys_exit, sys_write};

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub static __rust_no_alloc_shim_is_unstable_v2: u8 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let msg = b"Hello, RCore!\n";
    sys_write(1, msg.as_ptr(), msg.len());

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
            lateout("rax") _,
            lateout("rcx") _, // syscall 会破坏 rcx
            lateout("r11") _, // syscall 会破坏 r11
        );
    }

    sys_exit(0);
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
