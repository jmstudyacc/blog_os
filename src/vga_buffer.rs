// use this crate to prevent the compiler from erroneously optimising
// the compiler does not know that that the VGA buffer memory is being accessed
use volatile::Volatile;
// supports Rust formatting macros to easily print different types
use core::fmt;
use core::fmt::Arguments;

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
            self.buffer[row][col].write(blank);
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

pub fn print_test() {
    // a new writer is created that points to the VGA buffer at 0xb8000
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Green, Color::Black),
        // integer is cast as a mutable raw pointer and then converted to a mutable reference
        // by dereferencing it, with *, and immediately borrowing again
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    writer.write_byte(b'H');
    writer.write_string("ello ");
    writer.write_string("Wørld!");
    // call to unwrap() is needed as write! returns a Result
    // it would panic on any errors, but in this case, writes to VGA buffer never fail
    write!(writer, "The number are {} and {}", 42, 1.0 / 3.0).unwrap();
}
