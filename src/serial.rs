// in src/serial.rs

use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

// a lazy_static is used to ensure that the init is called only once
lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        // SerialPort::new expects the address of the first I/O port
        // it then calculates the addresses of all needed ports from this passed port
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };    // standard port number for first serial interface
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1
        .lock()
        .write_fmt(args)
        .expect("Printing to serial failed.")
}

// SerialPort provides its own implementation of fmt::Write trait
// it is not required to write our own implementation

// Prints to host through the serial interface & DOES NOT append new line
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    }
}

// Prints to host through serial interface & appends a new line
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(concat!($fmt, "\n"), $($arg)*));
}
