#![no_std]
#![no_main]

#[global_allocator]
static GLOBAL_ALLOCATOR: SimpleAllocator = SimpleAllocator::new();

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use new_rcore::allocator::SimpleAllocator;
use new_rcore::{sys_exit, sys_write};

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub static __rust_no_alloc_shim_is_unstable_v2: u8 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> () {
    let msg = b"Hello, RCore!\n";
    sys_write(1, msg.as_ptr(), msg.len());

    // 测试 1: 基本分配和释放
    let boxed = Box::new(42u32);
    let val = *boxed;
    assert_eq!(val, 42);
    drop(boxed);

    // 测试 2: Vec 分配
    let mut vec = Vec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    assert_eq!(vec.len(), 3);
    assert_eq!(vec[0], 1);
    assert_eq!(vec[1], 2);
    assert_eq!(vec[2], 3);

    // 测试 3: 重新分配
    let mut vec2 = Vec::with_capacity(4);
    for i in 0..10 {
        vec2.push(i);
    }

    let mut allocated_str: String = String::from("Hello, RCore Allocator!\n");
    assert_eq!(allocated_str, "Hello, RCore Allocator!\n");
    sys_write(1, allocated_str.as_ptr(), allocated_str.len());
    allocated_str += "This is a test of the allocator.\n";
    assert_eq!(
        allocated_str,
        "Hello, RCore Allocator!\nThis is a test of the allocator.\n"
    );
    sys_write(1, allocated_str.as_ptr(), allocated_str.len());

    sys_exit(0);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
