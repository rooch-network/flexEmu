use byteorder::ByteOrder;
use clap::Parser;

use omo::arch::{MipsProfile, MIPS};
use omo::config::OmoConfig;
use omo::errors::EmulatorError;
use omo::loader::ElfLoader;
use std::collections::HashMap;
use std::fs::read;
use std::path::PathBuf;

use omo::core::build_core;
use unicorn_engine::Unicorn;

#[derive(Parser)]
struct Options {
    #[clap(long = "config")]
    /// config file of the emulation.
    config_file: PathBuf,

    /// exec file
    exec: PathBuf,
    args: Vec<String>,
    // #[clap(long = "env")]
    // envs: Vec<(String, String)>,
}

fn main() -> Result<(), EmulatorError> {
    let opts: Options = Options::parse();

    let config: OmoConfig =
        toml::from_str(&std::fs::read_to_string(&opts.config_file).unwrap()).unwrap();
    let binary = read(opts.exec.as_path()).unwrap();
    let argv = {
        let mut a = opts.args.clone();
        a.insert(0, opts.exec.display().to_string());
        a
    };
    let mut uc: Unicorn<_> = {
        let arch = MIPS::new(MipsProfile::default());
        build_core(arch)
    };

    let load_result = ElfLoader::load(&config.os, binary.as_slice(), argv, &mut uc).unwrap();
    println!("load result: {:?}", &load_result);
    Ok(())
}

const TOTAL_MEMORY: usize = 0x180000000;
const UNTIL_PC: u64 = 0x5ead0004;
const REG_OFFSET: u32 = 0xc0000000;
const REG_PC: u32 = REG_OFFSET + 0x20 * 4;
const REG_HEAP: u32 = REG_OFFSET + 0x23 * 4;

/// length of data should be times of 4.
fn load_data(data: impl AsRef<[u8]>, ram: &mut HashMap<u32, u32>, base: u32) {
    let dat = data.as_ref();
    for (i, chunk) in dat.chunks(4).enumerate() {
        let value = byteorder::BigEndian::read_u32(chunk);
        if value != 0 {
            ram.insert(base + 4 * i as u32, value);
        }
    }
}

fn write_ram(ram: &mut HashMap<u32, u32>, addr: u32, value: u32) {
    ram.insert(addr, value);
}

fn write_bytes(_fd: u64, bytes: Vec<u8>) {
    eprint!("{}", String::from_utf8_lossy(&bytes));
}
