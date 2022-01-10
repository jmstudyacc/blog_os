// use this crate to prevent the compiler from erroneously optimising
// the compiler does not know that that the VGA buffer memory is being accessed
use volatile::Volatile;
// supports Rust formatting macros to easily print different types
use core::fmt;
use lazy_static::lazy_static;
// a simple mutex - the threads simply try to lock the mutex again & again in a tight loop
use spin::Mutex;

#[allow(dead_code)] // disables the compiler reporting unused enum variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)] // enables Copy semantics, printability and comparability
#[repr(u8)] // causes each enum variant to be stored as an u8 (Rust does not have a u4 type)
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] // causes the layout and ABI of the struct to be guaranteed to be the same as the single field
struct ColorCode(u8); // contains the full color byte, foreground & background

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // field ordering in default structs is undefined in Rust, this guarantees ordering like in a C struct
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    // use of the Volatile wrapper for Read & Write operations
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// construct required to track writing to the screen
pub struct Writer {
    // will always write to the last line & shift lines up when a line is full or \n
    column_position: usize,      // tracks the current position in the last row
    color_code: ColorCode,       // current fg & bg colors
    buffer: &'static mut Buffer, //reference to the VGA buffer is stored in Buffer (reference is valid for the entire runtime)
}

// as ColorCode::new causes the compiler to error, the const evaluator cannot convert raw pointers to references at compile time
// the lazy_static! crate & macro is used - the static lazily initializes itself when it is accessed for the first time
lazy_static! {
    // this WRITER is useless as it is immutable, 1 possible solution is use a mutable static (THIS IS HIGHLY DISCOURAGED)
    // an alternative to this is to use a Spinlock (spin@0.5.2)
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Green, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

impl Writer {
    // this will write a single ASCII byte
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(), // writer takes the new_line() action
            byte => {
                // checking if the current line is full
                if self.column_position >= BUFFER_WIDTH {
                    // add a new line
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                // the value in the buffer of [row][col] becomes a ScreenChar
                // use of the .write() method from Volatile ensure the compiler does not optimise this away
                self.buffer.chars[row][col].write(ScreenChar {
                    // the character written is found in the byte
                    ascii_character: byte,
                    // the color is found in the ColorCode field
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        // prints a whole string by converting to bytes & printing bytes one by one
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of the printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    // this function will move all the characters one line up & the top line is deleted
    // starting point, again, is the beginning of the last line
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    // Method clears a row by overwriting all of its characters with a space character
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

// to enable the Rust formatting macros we need to impl the trait on Writer
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// this attribute makes a macro available to the whole crate, not just the module
// it also requires the macro to be imported using std::print...
#[macro_export]
macro_rules! print {
    // this macro expands to a call to the _print function in the io module - it is at the root namespace
    // format_args! macro builds a fmt::Arguments type from passed arguments
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
#[macro_export]
pub fn _print(args: fmt::Arguments) {
    // write_fmt() is used so the Write trait needs to be imported
    use core::fmt::Write;
    // function locks the static WRITER and calls write_fmt() on it
    WRITER.lock().write_fmt(args).unwrap();
}
