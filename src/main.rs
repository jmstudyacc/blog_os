//! src/main.rs

// disables the Rust standard library link
#![no_std]
// disables the normal, main(), Rust entry point
#![no_main]

mod vga_buffer;

static HELLO: &[u8] = b"Hello World!";

// ensures Rust compiler outputs a function named _start and not a random name
#[no_mangle]
// 'extern "C"' is required so the compiler uses the C calling convention, not Rust's
pub extern "C" fn _start() -> ! {
    /*
    this function is the program entry point and should not return,
    instead should invoke the 'exit' system call of the OS
    */

    // casts the integer 0xb8000 into a raw pointer
    // let vga_buffer = 0xb8000 as *mut u8;
    //
    // // iterate over HELLO and enumerate() to get a running var i
    // for (i, &byte) in HELLO.iter().enumerate() {
    //     // unsafe block is used as Rust cannot verify the raw pointers are valid
    //     unsafe {
    //         // offset method is used to write the string byte and corresponding color byte
    //         *vga_buffer.offset(i as isize * 2) = byte;
    //         *vga_buffer.offset(i as isize * 2 + 1) = 0x2;
    //     }
    // }
    vga_buffer::print_test();
    loop {}
}

use core::panic::PanicInfo;

#[panic_handler]
/*
PanicInfo contains the file & line where the panic occurred & optional message
Divergent function returning a never type (!)
*/
fn panic(_info: &PanicInfo) -> ! {
    // function should just loop indefinitely for now
    loop {}
}

/*
panic_handler function is missing now the standard library has been unlinked
eh_personality language item is missing which disables the function for stack unwinding
instead Rust provides you with the option to abort on panic - check Cargo.toml
fn main() {
    //println!("Hello World!"); - println!() macro is a part of the standard library
 }
*/

// main() is removed as there is no crt0 available for the entry point
