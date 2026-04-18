use std::{ptr::write_volatile, thread};

fn main() {
    unsafe {
        let mut num = 0u32;
        let addr = (&raw mut num as *mut u32) as usize;
        let mut thread_vec = Vec::new();
        for _ in 0..10 {
            let handle = thread::spawn(move || unsafe {
                let num_addr = addr as *mut u32;
                for _ in 0..100_000 {
                    write_volatile(num_addr, *num_addr + 1);
                }
            });
            thread_vec.push(handle);
        }
        for handle in thread_vec {
            handle.join().unwrap();
        }
        println!("Final value of num: {}; expected: {}", num, 10 * 100_000);
    }
}
