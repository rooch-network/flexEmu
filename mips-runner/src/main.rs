use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::atomic::Ordering::Relaxed;
use byteorder::ByteOrder;
use unicorn_engine::{RegisterMIPS, Unicorn};
use unicorn_engine::unicorn_const::{Arch, Mode, Permission, uc_error};

use clap::Parser;

#[derive(Parser)]
struct Options {
    file: PathBuf,
    #[clap(long)]
    steps: u64,
}

fn main() -> Result<(), uc_error> {
    let options: Options = Options::parse();
    let binary = std::fs::read(options.file.as_path()).unwrap();
    let steps = options.steps;
    let mut ram = HashMap::<u32, u32>::new();
    let current_step = Arc::new(AtomicUsize::new(0));
    let heap_start = Arc::new(AtomicU64::new(0));
    let mut engine = Unicorn::new(Arch::MIPS,Mode::MIPS32 | Mode::BIG_ENDIAN).unwrap();
    engine.add_intr_hook({
        let current_step = current_step.clone();
        let heap_start = heap_start.clone();
        move |mu, intno|{
        if intno != 17 {
            eprintln!("invalid interrupt {} at step {}", intno, current_step.load(Ordering::Relaxed));
        }
        let syscall_no = mu.reg_read(RegisterMIPS::V0).unwrap();
        //println!("intr {} syscall {}", intno, syscall_no);
        let mut v0 = 0u64;
        match syscall_no {
            // println, i think
            4004 => {
                let fd = mu.reg_read(RegisterMIPS::A0).unwrap();
                let buf = mu.reg_read(RegisterMIPS::A1).unwrap();
                let count = mu.reg_read(RegisterMIPS::A2).unwrap();
                let bytes = mu.mem_read_as_vec(buf, count as usize).unwrap();
                write_bytes(fd, bytes);
            },
            4090 => {
                let a0 = mu.reg_read(RegisterMIPS::A0).unwrap();
                let sz = mu.reg_read(RegisterMIPS::A1).unwrap();
                if a0 == 0 {
                    v0 = 0x20000000 + heap_start.load(Ordering::Relaxed);
                    heap_start.fetch_add(sz, Ordering::Relaxed);
                } else {
                    v0 = a0;
                }
            }
            4045 => {
                v0 = 0x40000000;
            }
            4120 => {
                v0 = 1
            }
            4246 => mu.reg_write(RegisterMIPS::PC, 0x5ead0000).unwrap(),
            _ => {}
        };
        mu.reg_write(RegisterMIPS::V0, v0).unwrap();
        mu.reg_write(RegisterMIPS::A3, 0).unwrap();
    }})?;

    engine.add_code_hook(0, TOTAL_MEMORY as u64, {
        let current_step = current_step.clone();
        move |mu, addr, size| {
        let step = current_step.load(Relaxed) as u64;
        if step == steps {
            mu.reg_write(RegisterMIPS::PC, UNTIL_PC).unwrap();
        }
        // if step%1000 == 0 {
        println!("step: {}, addr: {}, size: {}", step, addr, size);
        // }
        current_step.fetch_add(1, Relaxed);
    }})?;
    engine.mem_map(0, TOTAL_MEMORY, Permission::all());


    engine.mem_write(0, &binary).unwrap();

    load_data(&binary, &mut ram, 0);

    engine.emu_start(0, UNTIL_PC, 0, 0).unwrap();
    Ok(())
}
const TOTAL_MEMORY: usize = 0x180000000;
const UNTIL_PC: u64 = 0x5ead0004;
const REG_OFFSET:u32 = 0xc0000000;
const REG_PC: u32 = REG_OFFSET + 0x20*4;
const REG_HEAP: u32 = REG_OFFSET + 0x23*4;

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

fn write_bytes(fd: u64, bytes: Vec<u8>) {
    eprint!("{}", String::from_utf8_lossy(&bytes));
}