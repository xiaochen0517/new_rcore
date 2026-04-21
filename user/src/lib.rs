#![no_std]
// #![no_main]
#![feature(linkage)]

#[macro_use]
pub mod console;
mod lang_items;
mod syscall;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    clear_bss();
    exit(main());
    panic!("Should never reach here!");
}

#[linkage = "weak"]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    panic!("main function not defined!");
}

fn clear_bss() {
    unsafe extern "C" {
        fn start_bss();
        fn end_bss();
    }
    (start_bss as *const () as usize..end_bss as *const () as usize)
        .for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

use syscall::*;

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}

pub fn exit(code: i32) -> isize {
    sys_exit(code)
}
