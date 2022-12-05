pub mod move_resources;
pub mod txn_builder;
use crate::{
    move_resources::{ChallengeData, Challenges},
    txn_builder::{confirm_state_transition, contain_state, defend_state},
};
use omo::{
    arch::mips::{MipsProfile, MIPS},
    config::OmoConfig,
    emulator::{Emulator, EmulatorState, StateChange},
    os::linux::LinuxRunner,
    step_proof::generate_step_proof,
};
use starcoin_crypto::{ed25519::Ed25519PrivateKey, HashValue, PrivateKey};
use starcoin_logger::prelude::info;
use starcoin_rpc_api::types::{ContractCall, TransactionStatusView};
use starcoin_rpc_client::{RemoteStateReader, RpcClient, StateRootOption};
use starcoin_types::{
    account_address::AccountAddress,
    account_config::AccountResource,
    transaction::{
        authenticator::{AccountPrivateKey, AuthenticationKey, AuthenticationKeyPreimage},
        RawUserTransaction, SignedUserTransaction, TransactionPayload,
    },
};
use starcoin_vm_types::state_view::StateReaderExt;
use std::{path::PathBuf, time::Duration};

pub struct SharedData {
    client: RpcClient,
    key: Ed25519PrivateKey,
    omo_config: OmoConfig,
    program: RunUnit,
}
pub struct Challenger {
    inner: SharedData,
    proposer_address: AccountAddress,
}

pub struct Proposer {
    fault_step: Option<u64>,
    inner: SharedData,
}
struct RunUnit {
    binary: Vec<u8>,
    argv: Vec<String>,
    env: Vec<(String, String)>,
}

impl Proposer {
    pub async fn run(
        &self,
        exec: Vec<u8>,
        argv: Vec<String>,
        envs: Vec<(String, String)>,
    ) -> anyhow::Result<()> {
        let inner = &self.inner;
        let emu_state = run_mips(inner.omo_config.clone(), exec, argv, envs, None)?;

        // declare state
        inner
            .client
            .submit_transaction(self.build_txn(txn_builder::declare_state(HashValue::new(
                emu_state.state_root(),
            )))?)?;

        let sender = AuthenticationKey::ed25519(&inner.key.public_key()).derived_address();
        loop {
            let remote_reader = inner.client.state_reader(StateRootOption::Latest)?;
            let challenges = remote_reader.get_resource::<Challenges>(sender)?.unwrap();
            for (id, c) in challenges.value.into_iter().enumerate() {
                self.handle_challenge(id as u64, c)?;
            }
            // sleep 3s
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
        Ok(())
    }
    async fn handle_challenge(&self, cid: u64, c: ChallengeData) -> anyhow::Result<()> {
        let me = self.inner.self_address();

        /// already stopped
        if c.l + 1 == c.r {
            let state_change = run_mips_state_change(
                self.inner.omo_config.clone(),
                self.inner.program.binary.clone(),
                self.inner.program.argv.clone(),
                self.inner.program.env.clone(),
                c.r as usize,
            )?;
            let state_proof = generate_step_proof(state_change);

            let txn = self.inner.build_txn(confirm_state_transition(
                me,
                cid,
                state_proof.access_nodes,
            ))?;
            let txn_hash = self.inner.client.submit_transaction(txn)?;
            let _ = self.inner.client.watch_txn(txn_hash, None)?;
            let txn_info = self
                .inner
                .client
                .chain_get_transaction_info(txn_hash)?
                .unwrap();
            info!("confirm_state_transition txn info: {:?}", txn_info);
            if txn_info.status == TransactionStatusView::Executed {
                info!(
                    "confirm_state_transition of {}-{} from  {} -> {}, with root {} -> {}",
                    me,
                    cid,
                    c.l,
                    c.r,
                    HashValue::new(state_proof.root_before),
                    HashValue::new(state_proof.root_after)
                );
            }
        } else {
            let next_step = (c.l + c.r) / 2;
            let r = self
                .inner
                .client
                .contract_call(contain_state(me, cid, next_step, self.defend))?
                .pop()
                .unwrap();
            let already_proposed = r.0.as_bool().unwrap();
            if !already_proposed {
                let state = run_mips(
                    self.inner.omo_config.clone(),
                    self.inner.program.binary.clone(),
                    self.inner.program.argv.clone(),
                    self.inner.program.env.clone(),
                    Some(next_step as usize),
                )?;
                let state_root = HashValue::new(state.state_root());
                let r = self
                    .inner
                    .client
                    .submit_transaction(self.build_txn(defend_state(cid, state_root))?)?;
                self.inner.client.watch_txn(r, None)?;
                info!("defend state {:?} at step {}", state_root, next_step);
            }
        }
        Ok(())
    }
}
impl SharedData {
    fn self_address(&self) -> AccountAddress {
        AuthenticationKey::ed25519(&self.key.public_key()).derived_address()
    }
    fn build_txn(&self, payload: TransactionPayload) -> anyhow::Result<SignedUserTransaction> {
        let sender = AuthenticationKey::ed25519(&self.key.public_key()).derived_address();
        let remote_reader = self.client.state_reader(StateRootOption::Latest)?;
        let txn = RawUserTransaction::new_with_default_gas_token(
            sender,
            remote_reader.get_sequence_number(sender)?,
            payload,
            200_000_000,
            1,
            remote_reader.get_timestamp()?.seconds() + 60 * 10,
            self.client.chain_id()?.id.into(),
        )
        .sign(&self.key, self.key.public_key())?
        .into_inner();
        Ok(txn)
    }
}

impl Challenger {
    pub async fn run(
        &self,
        exec: Vec<u8>,
        argv: Vec<String>,
        envs: Vec<(String, String)>,
    ) -> anyhow::Result<()> {
    }
}

fn run_mips(
    config: OmoConfig,
    binary: Vec<u8>,
    argv: Vec<String>,
    env: Vec<(String, String)>,
    until: Option<usize>, // stop step
) -> anyhow::Result<EmulatorState> {
    let mips_profile = MipsProfile::default();
    let arch = MIPS::new(mips_profile.pointer_size());
    let runner = LinuxRunner::new(config.os.mmap_address);
    let mut emu = Emulator::<_, LinuxRunner>::new(config, arch, mips_profile.mode(), runner)?;

    let load_info = emu.load(&binary, argv, env)?;

    let _total_steps = emu.run(load_info.entrypoint, None, None, until)?;
    let emu_state = emu.save()?;
    Ok(emu_state)
}

fn run_mips_state_change(
    config: OmoConfig,
    binary: Vec<u8>,
    argv: Vec<String>,
    env: Vec<(String, String)>,
    until: usize, // stop step
) -> anyhow::Result<StateChange> {
    let mips_profile = MipsProfile::default();
    let arch = MIPS::new(mips_profile.pointer_size());
    let runner = LinuxRunner::new(config.os.mmap_address);
    let mut emu = Emulator::<_, LinuxRunner>::new(config, arch, mips_profile.mode(), runner)?;

    let load_info = emu.load(&binary, argv, env)?;

    let state_change = emu.run_until(load_info.entrypoint, None, None, until)?;
    Ok(state_change)
}
