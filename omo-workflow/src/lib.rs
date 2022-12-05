pub mod move_resources;
pub mod txn_builder;
use crate::{
    move_resources::{ChallengeData, Challenges, Global},
    txn_builder::{
        assert_state, confirm_state_transition, contain_state, create_challenge, defend_state,
        deny_state_transition,
    },
};
use omo::{
    arch::mips::{MipsProfile, MIPS},
    config::OmoConfig,
    emulator::{Emulator, EmulatorState, StateChange},
    os::linux::LinuxRunner,
    step_proof::generate_step_proof,
};
use starcoin_crypto::{ed25519::Ed25519PrivateKey, HashValue, PrivateKey};
use starcoin_logger::prelude::{error, info};
use starcoin_rpc_api::types::{ContractCall, TransactionInfoView, TransactionStatusView};
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
        // exec: Vec<u8>,
        // argv: Vec<String>,
        // envs: Vec<(String, String)>,
    ) -> anyhow::Result<()> {
        let inner = &self.inner;
        let emu_state = run_mips(
            inner.omo_config.clone(),
            self.inner.program.binary.clone(),
            self.inner.program.argv.clone(),
            self.inner.program.envs.clone(),
            None,
        )?;

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
            } else {
                error!("confirm_state_transition failure with info: {:?}", txn_info);
            }
        } else {
            let next_step = (c.l + c.r) / 2;
            let r = self
                .inner
                .client
                .contract_call(contain_state(me, cid, next_step, true))?
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
    fn send_and_wait_txn(&self, txn: SignedUserTransaction) -> anyhow::Result<TransactionInfoView> {
        let txn_hash = self.client.submit_transaction(txn)?;
        let _ = self.client.watch_txn(txn_hash, None)?;
        let txn_info = self.client.chain_get_transaction_info(txn_hash)?.unwrap();
        Ok(txn_info)
    }
}

impl Challenger {
    pub async fn run(
        &self,
        // exec: Vec<u8>,
        // argv: Vec<String>,
        // envs: Vec<(String, String)>,
    ) -> anyhow::Result<()> {
        let self_address = self.inner.self_address();
        loop {
            let remote_reader = self.inner.client.state_reader(StateRootOption::Latest)?;
            let declared_state = remote_reader.get_resource::<Global>(self.proposer_address)?;
            if let Some(Global { declared_state }) = declared_state {
                let emu_state = run_mips(
                    self.inner.omo_config.clone(),
                    self.inner.program.binary.clone(),
                    self.inner.program.argv.clone(),
                    self.inner.program.envs.clone(),
                    None,
                )?;
                let my_state_root = HashValue::new(emu_state.state_root());
                if my_state_root != declared_state {
                    info!(
                        "found fraud of address {}, it root {} mismatched expected {}",
                        self.proposer_address, declared_state, my_state_root,
                    );
                    let txn = self.inner.build_txn(create_challenge(
                        self.proposer_address,
                        my_state_root,
                        emu_state.steps,
                    ))?;
                    let txn_info = self.inner.send_and_wait_txn(txn)?;
                    if txn_info.status == TransactionStatusView::Executed {
                        info!(
                            "create challenge success under {} at index {}",
                            self.proposer_address,
                            0 // TODO: change the hardcoded index
                        );
                    } else {
                        error!(
                            "create challenge failure, please check the code. txn info: {:?}",
                            txn_info
                        );
                        continue;
                    }

                    loop {
                        let remote_reader =
                            self.inner.client.state_reader(StateRootOption::Latest)?;
                        let challenges = remote_reader.get_resource::<Challenges>(sender)?.unwrap();
                        for (id, c) in challenges.value.into_iter().enumerate() {
                            self.handle_challenge(id as u64, c)?;
                        }
                        // sleep 3s
                        tokio::time::sleep(Duration::from_secs(3)).await;
                    }
                }
            }
        }
    }
    fn handle_challenge(&self, cid: u64, c: ChallengeData) -> anyhow::Result<()> {
        let me = self.inner.self_address();
        let proposer_address = self.proposer_address;
        if c.l + 1 == c.r {
            let state_change = run_mips_state_change(
                self.inner.omo_config.clone(),
                self.inner.program.binary.clone(),
                self.inner.program.argv.clone(),
                self.inner.program.env.clone(),
                c.r as usize,
            )?;
            let state_proof = generate_step_proof(state_change);
            let txn =
                self.inner
                    .build_txn(deny_state_transition(me, cid, state_proof.access_nodes))?;
            let txn_info = self.inner.send_and_wait_txn(txn)?;
            if txn_info.status == TransactionStatusView::Executed {
                info!(
                    "deny_state_transition of {}-{} from  {} -> {}, with root {} -> {}",
                    me,
                    cid,
                    c.l,
                    c.r,
                    HashValue::new(state_proof.root_before),
                    HashValue::new(state_proof.root_after)
                );
            } else {
                error!("deny_state_transition failure with info: {:?}", txn_info);
            }
        } else {
            let next_step = (c.l + c.r) / 2;
            let r = self
                .inner
                .client
                .contract_call(contain_state(proposer_address, cid, next_step, true))?
                .pop()
                .unwrap();
            let defended = r.0.as_bool().unwrap();

            if !defended {
                info!("waiting defender's response at step {}", next_step);
            } else {
                let asserted = self
                    .inner
                    .client
                    .contract_call(contain_state(proposer_address, cid, next_step, false))?
                    .pop()
                    .unwrap()
                    .0
                    .as_bool()
                    .unwrap();
                if asserted {
                    info!(
                        "already asserted at step {}, wait for defender's next response",
                        next_step
                    );
                } else {
                    let state = run_mips(
                        self.inner.omo_config.clone(),
                        self.inner.program.binary.clone(),
                        self.inner.program.argv.clone(),
                        self.inner.program.env.clone(),
                        Some(next_step as usize),
                    )?;
                    let state_root = HashValue::new(state.state_root());
                    let txn =
                        self.inner
                            .build_txn(assert_state(proposer_address, cid, state_root))?;
                    let txn_info = self.inner.send_and_wait_txn(txn)?;
                    info!("assert state {:?} at step {}", state_root, next_step);
                }
            }
        }
        Ok(())
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
