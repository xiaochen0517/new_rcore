use log::{debug, info};

use crate::batch::run_next_app;

pub fn sys_exit(exit_code: isize) -> ! {
    debug!("[syscall] Exit with code {}", exit_code);
    run_next_app()
}
