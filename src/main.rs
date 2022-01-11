//! src/main.rs

// disables the Rust standard library link
#![no_std]
// disables the normal, main(), Rust entry point
#![no_main]
// unstable feature that requires no external libraries - collects all functions annotated with #[test_case]
#![feature(custom_test_frameworks)]
// defines the test runner to be used
// unfortunately this approach loses some advanced features e.g. #[should_panic] - this must be handwritten
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

//static HELLO: &[u8] = b"Hello World!";

use blog_os::println;
use core::panic::PanicInfo;

// ensures Rust compiler outputs a function named _start and not a random name
#[no_mangle]
// 'extern "C"' is required so the compiler uses the C calling convention, not Rust's
pub extern "C" fn _start() -> ! {
    /*
    this function is the program entry point and should not return,
    instead should invoke the 'exit' system call of the OS
    */
    println!("Hello World{}", "!");

    // due to the way _start() is coded once the test ends an infinite loop starts - cargo never returns
    #[cfg(test)]
    test_main();

    loop {}
}

// new attribute that runs when NOT in Cargo Test
#[cfg(not(test))]
#[panic_handler]
/*
PanicInfo contains the file & line where the panic occurred & optional message
Divergent function returning a never type (!)
*/
fn panic(info: &PanicInfo) -> ! {
    // function should just loop indefinitely for now
    // to include the panic! messages println would need to change to serial_println when testing
    println!("{}", info);
    loop {}
}

// attribute to denote running when using cargo test
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}
/*
panic_handler function is missing now the standard library has been unlinked
eh_personality language item is missing which disables the function for stack unwinding
instead Rust provides you with the option to abort on panic - check Cargo.toml
*/

// main() is removed as there is no crt0 available for the entry point

// creating a test case that passes and prints to terminal
#[test_case]
fn trivial_assertion() {
    // the serial_prints can be removed from here as they are captured in the Testable implementation
    //serial_print!("trivial assertion...");
    assert_eq!(1, 1);
    //serial_println!("[OK]");
}
