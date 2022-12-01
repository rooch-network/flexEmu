use omo::{
    arch::mips::{MipsProfile, MIPS},
    config::OmoConfig,
    emulator::Emulator,
    os::linux::LinuxRunner,
};
use starcoin_rpc_client::RpcClient;
use starcoin_types::transaction::authenticator::AccountPrivateKey;
use std::path::PathBuf;

pub struct Roler {
    client: RpcClient,
    key: AccountPrivateKey,
    defend: bool,
    fault_step: Option<u64>,
    omo_config: OmoConfig,
}

impl Roler {
    pub async fn run(
        self,
        exec: Vec<u8>,
        argv: Vec<String>,
        envs: Vec<(String, String)>,
    ) -> anyhow::Result<()> {
        let total_count = run_mips(self.omo_config.clone(), exec, argv, envs)?;
    }
}

fn run_mips(
    config: OmoConfig,
    binary: Vec<u8>,
    argv: Vec<String>,
    env: Vec<(String, String)>,
) -> anyhow::Result<u64> {
    let mips_profile = MipsProfile::default();
    let arch = MIPS::new(mips_profile.pointer_size());
    let runner = LinuxRunner::new(config.os.mmap_address);
    let mut emu = Emulator::<_, LinuxRunner>::new(config, arch, mips_profile.mode(), runner)?;

    let load_info = emu.load(&binary, argv, env)?;
    Ok(emu.run(load_info.entrypoint, None, None, None)?)
}
