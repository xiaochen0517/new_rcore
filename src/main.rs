#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
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
