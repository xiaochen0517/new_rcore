#![no_main]
#![no_std]

#[macro_use]
extern crate user_lib;

#[unsafe(no_mangle)]
fn main() -> i32 {
    println!("Hello, world!");
    0
}
