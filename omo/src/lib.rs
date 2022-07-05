#![feature(mixed_integer_ops)]
#![feature(generic_const_exprs)]
#![feature(adt_const_params)]

pub mod arch;
pub mod cc;
pub mod config;
pub mod core;
pub mod data;
pub mod emulator;
pub mod errors;
pub mod loader;
pub mod memory;
pub mod os;
pub mod registers;
pub mod stack;
pub mod utils;
pub const PAGE_SIZE: u32 = 0x1000;
