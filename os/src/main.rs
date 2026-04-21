#![no_std]
#![no_main]
// #![feature(panic_info_message)]

use core::arch::global_asm;
use log::*;

#[macro_use]
mod console;
mod lang_items;
mod logging;
mod sbi;

#[path = "boards/qemu.rs"]
mod board;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    extern "C" {
        fn stext(); // begin addr of text segment
        fn etext(); // end addr of text segment
        fn srodata(); // start addr of Read-Only data segment
        fn erodata(); // end addr of Read-Only data ssegment
        fn sdata(); // start addr of data segment
        fn edata(); // end addr of data segment
        fn sbss(); // start addr of BSS segment
        fn ebss(); // end addr of BSS segment
        fn boot_stack_lower_bound(); // stack lower bound
        fn boot_stack_top(); // stack top
    }
    clear_bss();
    logging::init();
    println!("[kernel] Hello, world!");
    trace!(
        "[kernel] .text [{:#x}, {:#x})",
        stext as *const () as usize,
        etext as *const () as usize
    );
    debug!(
        "[kernel] .rodata [{:#x}, {:#x})",
        srodata as *const () as usize, erodata as *const () as usize
    );
    info!(
        "[kernel] .data [{:#x}, {:#x})",
        sdata as *const () as usize, edata as *const () as usize
    );
    warn!(
        "[kernel] boot_stack top=bottom={:#x}, lower_bound={:#x}",
        boot_stack_top as *const () as usize, boot_stack_lower_bound as *const () as usize
    );
    error!(
        "[kernel] .bss [{:#x}, {:#x})",
        sbss as *const () as usize, ebss as *const () as usize
    );

    use crate::board::QEMUExit;
    crate::board::QEMU_EXIT_HANDLE.exit_success()
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as *const () as usize..ebss as *const () as usize)
        .for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}
