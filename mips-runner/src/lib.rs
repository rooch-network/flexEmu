#![feature(mixed_integer_ops)]

pub mod errors;
pub mod arch;
pub mod registers;
pub mod memory;
pub mod cc;
pub mod loader;
pub mod utils;
pub mod emulator;
pub mod os;
pub const PAGE_SIZE: u32 = 0x1000;

