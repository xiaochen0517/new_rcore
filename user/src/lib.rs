#![no_std]
#![no_main]
#![feature(linkage)]

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    clear_bss();
    // exit(main());
    panic!("Should never reach here!");
}

#[linkage = "weak"]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    panic!("main function not defined!");
}

fn clear_bss() {
    unsafe extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as *const () as usize..ebss as *const () as usize)
        .for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}
