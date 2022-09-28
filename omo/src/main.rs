use byteorder::ByteOrder;
use clap::Parser;
use log::{info, LevelFilter};
use omo::{
    arch::mips::{MipsProfile, MIPS},
    config::OmoConfig,
    emulator::Emulator,
    errors::EmulatorError,
    os::linux::LinuxRunner,
};
use std::{
    collections::HashMap,
    env,
    error::Error,
    fs::{create_dir_all, read},
    num::NonZeroUsize,
    path::PathBuf,
};

#[derive(Parser)]
struct Options {
    #[clap(long = "config")]
    /// config file of the emulation.
    config_file: PathBuf,
    #[clap(subcommand)]
    command: SubCommands,
}

#[derive(Parser)]
enum SubCommands {
    Run {
        /// exec file
        exec: PathBuf,
        args: Vec<String>,
        #[clap(long = "env", parse(try_from_str=parse_key_val))]
        envs: Vec<(String, String)>,
    },
    GenState {
        /// exec file
        exec: PathBuf,
        args: Vec<String>,
        #[clap(long = "env", parse(try_from_str=parse_key_val))]
        envs: Vec<(String, String)>,
        /// gen state before/after executing N steps.
        #[clap(short, long)]
        steps: NonZeroUsize,
        /// save the states under output_dir.
        #[clap(short, long)]
        output_dir: Option<PathBuf>,
    },
}

fn main() -> Result<(), EmulatorError> {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    let opts: Options = Options::parse();

    let config: OmoConfig =
        toml::from_str(&std::fs::read_to_string(&opts.config_file).unwrap()).unwrap();

    match opts.command {
        SubCommands::Run { exec, args, envs } => {
            let binary = read(exec.as_path()).unwrap();
            let argv = {
                let mut a = args;
                a.insert(0, exec.display().to_string());
                a
            };
            let env = envs;

            let mips_profile = MipsProfile::default();
            let arch = MIPS::new(mips_profile.pointer_size());
            let runner = LinuxRunner::new();
            let mut emu =
                Emulator::<_, LinuxRunner>::new(config, arch, mips_profile.mode(), runner)?;

            let load_info = emu.load(&binary, argv, env)?;
            info!("load info: {:?}", &load_info);

            emu.run(load_info.entrypoint, None, None, None)?;
        }
        SubCommands::GenState {
            exec,
            args,
            envs,
            output_dir,
            steps,
        } => {
            let binary = read(exec.as_path()).unwrap();
            let argv = {
                let mut a = args;
                a.insert(0, exec.display().to_string());
                a
            };
            let env = envs;

            let mips_profile = MipsProfile::default();
            let arch = MIPS::new(mips_profile.pointer_size());
            let runner = LinuxRunner::new();
            let mut emu =
                Emulator::<_, LinuxRunner>::new(config, arch, mips_profile.mode(), runner)?;

            let load_info = emu.load(&binary, argv, env)?;
            info!("load info: {:?}", &load_info);

            let state_change = emu.run_until(load_info.entrypoint, None, None, steps.get() - 1)?;
            let output_dir = output_dir
                .unwrap_or(env::current_dir().unwrap())
                .join(format!("step-{}", steps));
            create_dir_all(&output_dir).unwrap();
            serde_json::to_writer_pretty(
                std::fs::File::options()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(output_dir.join("before_state.json"))
                    .unwrap(),
                &state_change.state_before,
            )
            .unwrap();
            serde_json::to_writer_pretty(
                std::fs::File::options()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(output_dir.join("after_state.json"))
                    .unwrap(),
                &state_change.state_after,
            )
            .unwrap();
            serde_json::to_writer_pretty(
                std::fs::File::options()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(output_dir.join("readset.json"))
                    .unwrap(),
                &state_change.readset,
            )
            .unwrap();
            serde_json::to_writer_pretty(
                std::fs::File::options()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(output_dir.join("writeset.json"))
                    .unwrap(),
                &state_change.writeset,
            )
            .unwrap();
        }
    }
    Ok(())
}

/// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), anyhow::Error>
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
