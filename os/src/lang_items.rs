//! The panic handler

use crate::sbi::shutdown;
use core::panic::PanicInfo;

#[panic_handler]
/// panic handler
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "[kernel] Panicked at {}:{} {:?}",
            location.file(),
            location.line(),
            info.message().as_str().unwrap_or("No message")
        );
    } else {
        println!(
            "[kernel] Panicked: {:?}",
            info.message().as_str().unwrap_or("No message")
        );
    }
    shutdown()
}
