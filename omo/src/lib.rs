#![feature(mixed_integer_ops)]
//#![feature(generic_const_exprs)]
//#![feature(adt_const_params)]
#![feature(box_syntax)]
#![feature(slice_as_chunks)]

extern crate core;

use std::error::Error;

pub mod arch;
pub mod cc;
pub mod config;
pub mod emulator;
pub mod engine;
pub mod errors;
pub mod loader;
pub mod memory;
pub mod os;
pub mod rand;
pub mod registers;
pub mod stack;
pub mod utils;

pub const PAGE_SIZE: u32 = 0x1000;

pub mod step_proof;

/// Parse a single key-value pair
pub fn parse_key_val<T, U>(s: &str) -> Result<(T, U), anyhow::Error>
where
    T: std::str::FromStr,
    T::Err: Error + 'static + Sync + Send,
    U: std::str::FromStr,
    U::Err: Error + 'static + Sync + Send,
{
    let pos = s
        .find('=')
        .ok_or_else(|| anyhow::anyhow!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}
