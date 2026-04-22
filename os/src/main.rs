#![no_std]
#![no_main]

use core::arch::global_asm;
use log::*;

#[macro_use]
mod console;
mod batch;
mod lang_items;
mod logging;
mod sbi;
mod sync;
mod syscall;
mod trap;

#[path = "boards/qemu.rs"]
mod board;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("app_linker.S"));

#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    unsafe extern "C" {
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
    info!("[main] rCore is booting...");
    trap::init();
    info!("[main] Trap handler initialized.");
    batch::init();
    info!("[main] Batch kernel initialization complete.");
    batch::run_next_app();

    use crate::board::QEMUExit;
    crate::board::QEMU_EXIT_HANDLE.exit_success()
}

fn clear_bss() {
    unsafe extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as *const () as usize..ebss as *const () as usize)
        .for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}
