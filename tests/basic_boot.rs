// in tests/basic_boot.rs

// all integration tests are their own executables and completely separate from main.rs
// each tests needs to define its own entry point function
// all the crate attributes will need to be provided again - it is almost like a copy of main.rs
// the attribute cfg(test) is not needed as integration tests are ONLY built in test mode

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(blog_os::test_runner)]

// importing the following items from the blog_os crate
use blog_os::println;
use core::panic::PanicInfo;

#[no_mangle] //do not mangle the function name
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info);
    loop {}
}

#[test_case]
fn test_println() {
    println!("test_println output");
}
