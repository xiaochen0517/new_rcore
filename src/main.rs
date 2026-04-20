fn main() {
    println!("Hello, world!");
}

mod tests {

    use std::{ptr::write_volatile, sync::atomic::AtomicU32, sync::atomic::Ordering, thread};

    #[test]
    fn test_concurrent_error() {
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
            assert_eq!(num, 10 * 100_000);
        }
    }

    #[test]
    fn test_concurrent_fixed() {
        static NUM: AtomicU32 = AtomicU32::new(0);
        NUM.store(0, Ordering::SeqCst);
        let mut thread_vec = Vec::new();
        for _ in 0..10 {
            let handle = thread::spawn(move || {
                for _ in 0..100_000 {
                    NUM.fetch_add(1, Ordering::SeqCst);
                }
            });
            thread_vec.push(handle);
        }
        for handle in thread_vec {
            handle.join().unwrap();
        }
        let final_value = NUM.load(Ordering::SeqCst);
        println!(
            "Final value of num: {}; expected: {}",
            final_value,
            10 * 100_000
        );
        assert_eq!(final_value, 10 * 100_000);
    }
}
