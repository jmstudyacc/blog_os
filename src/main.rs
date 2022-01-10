//! src/main.rs

// disables the Rust standard library link
#![no_std]
// disables the normal, main(), Rust entry point
#![no_main]
// unstable feature that requires no external libraries - collects all functions annotated with #[test_case]
#![feature(custom_test_frameworks)]
// defines the test runner to be used
// unfortunately this approach loses some advanced features e.g. #[should_panic] - this must be handwritten
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod serial;
mod vga_buffer;

static HELLO: &[u8] = b"Hello World!";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)] // represents each variant by a u32 integer
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

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

use crate::vga_buffer::Writer;
use core::panic::PanicInfo;

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
    // serial_println is used here as opposed to println to ensure the message exits QEMU
    serial_println!("[FAILED]\n");
    serial_println!("ERROR: {}\n", info);
    // the QemuExitCode::Failed it provided to end the running instance of the OS
    exit_qemu(QemuExitCode::Failed);
    // infinite loop is still required as the compiler does not know isa-debug-exit causes program exit
    loop {}
}
/*
panic_handler function is missing now the standard library has been unlinked
eh_personality language item is missing which disables the function for stack unwinding
instead Rust provides you with the option to abort on panic - check Cargo.toml
*/

// main() is removed as there is no crt0 available for the entry point

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    // this is unsafe as writing to an I/O port can generally result in arbitrary behaviour
    unsafe {
        // creates a new port at 0xf4 - the iobase of the isa-debug-exit device
        let mut port = Port::new(0xf4);
        // writes the exit code, passed to the function, into the port - as u32 as exit is 0x04
        port.write(exit_code as u32);
    }
}

// attribute to mark the function as a test - as defined at the start of the code
#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    // a short debug message is printed
    println!("Running {} tests", tests.len());
    // each function in the list, tests, is called
    // this is like a list of references to types that can be called like a function
    for test in tests {
        test();
    }

    // enabling the test_runner to exit QEMU once all tests are complete
    exit_qemu(QemuExitCode::Success);
}

// creating a test case that passes and prints to terminal
#[test_case]
fn trivial_assertion() {
    //print!("trivial assertion...");
    // println!("[OK]");
    // As the SerialPort is now implemented we can print to serial
    // serial_println lives under the root namespace due to the #[macro_export] attribute
    serial_print!("trivial assertion...");
    assert_eq!(1, 1);
    serial_println!("[OK]");
}

#[test_case]
fn failed_assertion() {
    // following print statements are output to serial interface
    serial_print!("failing assertion...");
    assert_eq!(1, 0);
    serial_println!("[FAILED]");
}
