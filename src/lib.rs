#![no_std]
#![no_main]

pub mod allocator;

extern crate alloc;

pub fn sys_write(fd: usize, buf: *const u8, count: usize) -> isize {
    let mut s = 0isize;
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") 1usize, // sys_write
            in("rdi") fd,
            in("rsi") buf,
            in("rdx") count,
            lateout("rax") s,
            lateout("rcx") _, // syscall 会破坏 rcx
            lateout("r11") _, // syscall 会破坏 r11
        );
    }
    s
}

pub fn sys_exit(code: usize) -> ! {
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") 60usize, // sys_exit
            in("rdi") code,
            options(noreturn),
        );
    }
}
