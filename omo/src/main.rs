use byteorder::ByteOrder;
use clap::Parser;
use omo::{
    arch::mips::{MipsProfile, MIPS},
    config::OmoConfig,
    emulator::Emulator,
    errors::EmulatorError,
    os::linux::LinuxRunner,
};
use std::{collections::HashMap, fs::read, path::PathBuf};

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
    env_logger::init();
    let opts: Options = Options::parse();

    let config: OmoConfig =
        toml::from_str(&std::fs::read_to_string(&opts.config_file).unwrap()).unwrap();
    let binary = read(opts.exec.as_path()).unwrap();
    let argv = {
        let mut a = opts.args.clone();
        a.insert(0, opts.exec.display().to_string());
        a
    };
    let mips_profile = MipsProfile::default();
    let arch = MIPS::new(mips_profile.pointer_size());
    let runner = LinuxRunner::new();
    let mut emu = Emulator::<_, LinuxRunner>::new(config, arch, mips_profile.mode(), runner)?;

    let load_info = emu.load(&binary, argv)?;

    // let mut uc: Unicorn<_> = {
    //     let arch = ;
    //     build_core(arch)
    // };

    emu.run(load_info.entrypoint, None, None, None)?;
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
