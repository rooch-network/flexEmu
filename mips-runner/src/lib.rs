#![feature(mixed_integer_ops)]

pub mod arch;
pub mod cc;
pub mod data;
pub mod emulator;
pub mod errors;
pub mod loader;
pub mod memory;
pub mod os;
pub mod registers;
pub mod utils;
pub const PAGE_SIZE: u32 = 0x1000;
