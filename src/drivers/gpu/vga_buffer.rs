//! this is a simple vga buffer driver

use core::fmt;
use core::ops::{Deref, DerefMut};
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use x86::io::outb;

/// vga text mode buffer width
const BUFFER_WIDTH: usize = 80;
/// vga text mode buffer height
const BUFFER_HEIGHT: usize = 25;
/// vga buffer memory address
const VGA_BUFFER_ADDR: usize = 0xb8000;
const VGA_INDEX_REGISTER: u16 = 0x3D4;
const VGA_DATA_REGISTER: u16 = 0x3D5;

/// some color
#[repr(u8)]
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Eq, PartialOrd, PartialEq)]
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

/// vga buffer color foreground color and background color
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VgaColor(u8);

impl VgaColor {
    fn new(foreground: Color, background: Color) -> VgaColor {
        VgaColor((background as u8) << 4 | (foreground as u8))
    }
}

impl From<VgaColor> for u8 {
    fn from(value: VgaColor) -> Self {
        value.0
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VgaChar {
    ascii_chara: u8,
    color_code: VgaColor,
}

impl Deref for VgaChar {
    type Target = VgaChar;

    fn deref(&self) -> &Self::Target {
        &self
    }
}

impl DerefMut for VgaChar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut (*self)
    }
}

/// vga text buffer
#[repr(transparent)]
struct VgaBuffer {
    /// vga buffer
    chars: [[Volatile<VgaChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// char writer
pub struct Writer {
    color_code: VgaColor,
    col_position: usize,
    row_position: usize,
    cursor_position: u16,
    // 静态 可变
    buffer: &'static mut VgaBuffer,
}

impl Writer {
    pub fn new() -> Self {
        let mut writer = Writer {
            col_position: 0,
            row_position: 0,
            cursor_position: 0,
            color_code: VgaColor::new(Color::Yellow, Color::Black),
            buffer: unsafe { &mut *(VGA_BUFFER_ADDR as *mut VgaBuffer) },
        };

        // 清空屏幕
        writer.clear();

        writer
    }

    /// 按字节写入
    pub fn write_byte(&mut self, byte: u8) {
        let mut row = self.row_position;
        let mut col = self.col_position;
        let color_code = self.color_code;

        match byte {
            b'\n' => self.new_line(),
            byte => {
                // auto wrap 自动换行逻辑
                if self.col_position > BUFFER_WIDTH - 1 {
                    self.new_line();
                    row = self.row_position;
                    col = self.col_position;
                }
                // 写入字符
                self.buffer.chars[row][col].write(VgaChar {
                    ascii_chara: byte,
                    color_code,
                });
                // 维护指针索引+1
                self.col_position += 1;

                // 更新指针
                self.update_cursor();
            }
        }
    }

    /// 打印字符串,不可见字符统一用0xfe代替
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// 换行逻辑
    pub fn new_line(&mut self) {
        if self.row_position < BUFFER_HEIGHT - 1 {
            self.row_position += 1;
        } else {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = self.buffer.chars[row][col].read();
                    // 倒腾字符
                    self.buffer.chars[row - 1][col].write(character);
                }
            }
            // 清空最后一行
            self.clear_row(BUFFER_HEIGHT - 1);
        }

        // 字符指针归位
        self.col_position = 0;
        self.update_cursor();
    }

    /// 清空某一行
    fn clear_row(&mut self, row: usize) {
        let blank = VgaChar {
            ascii_chara: b' ',
            color_code: self.color_code,
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    fn update_cursor(&mut self) {
        self.cursor_position = (self.row_position * BUFFER_WIDTH + self.col_position) as u16;
        unsafe {
            outb(VGA_INDEX_REGISTER, 0x0F);
            outb(VGA_DATA_REGISTER, (self.cursor_position & 0xFF) as u8);
            outb(VGA_INDEX_REGISTER, 0x0E);
            outb(VGA_DATA_REGISTER, ((self.cursor_position >> 8) & 0xFF) as u8);
        }
    }

    pub fn clear(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row)
        }
        self.col_position = 0;
        self.row_position = 0;
        self.update_cursor();
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// 全局唯一统一入口
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer::new());
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::drivers::gpu::vga_buffer::_print(format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}