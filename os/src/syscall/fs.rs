use core::slice::from_raw_parts;
use core::{panic, str::from_utf8};

use log::debug;

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    debug!(
        "[syscall] sys_write called with fd={}, buf={:#x}, len={}",
        fd, buf as usize, len
    );
    match fd {
        FD_STDOUT => {
            let slice = unsafe { from_raw_parts(buf, len) };
            let str = from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
        }
        _ => panic!("Unsupported fd: {}", fd),
    }
}
