use core::arch::asm;

const SBI_CONSOLE_PUTCHAR: usize = 1;

#[inline(always)]
fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") arg0 => ret,
            in("x11") arg1,
            in("x12") arg2,
            in("x16") 0usize,
            in("x17") which,
        );
    }
    ret
}

pub fn console_putchar(c: usize) {
    sbi_call(SBI_CONSOLE_PUTCHAR, c, 0, 0);
}

use crate::board::QEMUExit;

pub fn shutdown(success: bool) -> ! {
    if success {
        crate::board::QEMU_EXIT_HANDLE.exit_success()
    } else {
        crate::board::QEMU_EXIT_HANDLE.exit_failure()
    }
}
