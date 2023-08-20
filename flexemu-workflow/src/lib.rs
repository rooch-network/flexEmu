pub mod move_resources;
pub mod txn_builder;

use crate::{
    move_resources::{ChallengeData, Challenges, Global},
    txn_builder::{
        assert_state, confirm_state_transition, contain_state, create_challenge, defend_state,
        deny_state_transition,
    },
};
use log::{error, info};
use flexemu::{
    arch::mips::{MipsProfile, MIPS},
    config::FlexEmuConfig,
    emulator::{Emulator, EmulatorState, StateChange},
    os::linux::LinuxRunner,
    step_proof::generate_step_proof,
};
use starcoin_crypto::{ed25519::Ed25519PrivateKey, HashValue, PrivateKey};
use starcoin_rpc_api::types::{TransactionInfoView, TransactionStatusView};
use starcoin_rpc_client::{RpcClient, StateRootOption};
use starcoin_types::{
    account_address::AccountAddress,
    transaction::{
        authenticator::AuthenticationKey, RawUserTransaction, SignedUserTransaction,
        TransactionPayload,
    },
};
use starcoin_vm_types::state_view::StateReaderExt;
use std::{thread::sleep, time::Duration};

pub struct SharedData {
    client: RpcClient,
    key: Ed25519PrivateKey,
    flexemu_config: FlexEmuConfig,
    program: RunUnit,
}
pub struct Challenger {
    inner: SharedData,
    proposer_address: AccountAddress,
}

pub struct Proposer {
    fault_step: Option<usize>,
    inner: SharedData,
}
pub struct RunUnit {
    pub binary: Vec<u8>,
    pub argv: Vec<String>,
    pub env: Vec<(String, String)>,
}

impl Proposer {
    pub fn new(
        flexemu_config: FlexEmuConfig,
        program: RunUnit,
        key: Ed25519PrivateKey,
        rpc: RpcClient,
        fault_step: Option<usize>,
    ) -> Self {
        Self {
            fault_step,
            inner: SharedData {
                client: rpc,
                key,
                program,
                flexemu_config,
            },
        }
    }
    pub fn run(
        &self,
        // exec: Vec<u8>,
        // argv: Vec<String>,
        // envs: Vec<(String, String)>,
    ) -> anyhow::Result<()> {
        let inner = &self.inner;

        // declare state
        if let Some(g) = inner
            .client
            .state_reader(StateRootOption::Latest)?
            .get_resource::<Global>(self.inner.self_address())?
        {
            info!("already declare_state with root {}", g.declared_state);
        } else {
            let emu_state = run_mips(
                self.inner.flexemu_config.clone(),
                self.inner.program.binary.clone(),
                self.inner.program.argv.clone(),
                self.inner.program.env.clone(),
                self.fault_step,
            )?;

            let root = HashValue::new(emu_state.state_root());
            self.inner
                .send_and_wait_txn(self.inner.build_txn(txn_builder::declare_state(root))?)?;
            info!("declare_state with root {}", root);
        }

        loop {
            let remote_reader = inner.client.state_reader(StateRootOption::Latest)?;
            let challenges = remote_reader
                .get_resource::<Challenges>(self.inner.self_address())?
                .unwrap();
            for (id, c) in challenges.value.into_iter().enumerate() {
                self.handle_challenge(id as u64, c)?;
            }
            // sleep 3s

            sleep(Duration::from_secs(3));
        }
    }
    fn handle_challenge(&self, cid: u64, c: ChallengeData) -> anyhow::Result<()> {
        if c.success != 0 {
            return Ok(());
        }
        let me = self.inner.self_address();

        // already stopped
        if c.l + 1 == c.r {
            let state_change = run_mips_state_change(
                self.inner.flexemu_config.clone(),
                self.inner.program.binary.clone(),
                self.inner.program.argv.clone(),
                self.inner.program.env.clone(),
                c.l as usize,
            )?;
            let state_proof = generate_step_proof(state_change);

            let txn = self.inner.build_txn(confirm_state_transition(
                me,
                cid,
                state_proof.access_nodes,
            ))?;
            let txn_info = self.inner.send_and_wait_txn(txn)?;
            info!(
                "try confirm_state_transition of {}-{} from  {} -> {}, with root {} -> {}",
                me,
                cid,
                c.l,
                c.r,
                HashValue::new(state_proof.root_before),
                HashValue::new(state_proof.root_after)
            );

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
            let asserted = self
                .inner
                .client
                .contract_call(contain_state(me, cid, next_step, false))?
                .pop()
                .unwrap()
                .0
                .as_bool()
                .unwrap();
            if !asserted {
                info!("wait for challenger's assertion at step {}", next_step);
            } else {
                let already_proposed = self
                    .inner
                    .client
                    .contract_call(contain_state(me, cid, next_step, true))?
                    .pop()
                    .unwrap()
                    .0
                    .as_bool()
                    .unwrap();
                if already_proposed {
                    info!("already defend state at step {}", next_step);
                } else {
                    let state_root = match self.fault_step {
                        Some(f) if f <= next_step as usize => HashValue::new(
                            run_mips(
                                self.inner.flexemu_config.clone(),
                                self.inner.program.binary.clone(),
                                self.inner.program.argv.clone(),
                                self.inner.program.env.clone(),
                                Some(f),
                            )?
                            .state_root(),
                        ),
                        _ => HashValue::new(
                            run_mips(
                                self.inner.flexemu_config.clone(),
                                self.inner.program.binary.clone(),
                                self.inner.program.argv.clone(),
                                self.inner.program.env.clone(),
                                Some(next_step as usize),
                            )?
                            .state_root(),
                        ),
                    };

                    let r = self
                        .inner
                        .send_and_wait_txn(self.inner.build_txn(defend_state(cid, state_root))?)?;
                    if r.status == TransactionStatusView::Executed {
                        info!("defend state {:?} at step {}", state_root, next_step);
                    } else {
                        error!("defend state failure due to {:?}", r);
                    }
                }
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
        let sender = self.self_address();
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
    pub fn new(
        flexemu_config: FlexEmuConfig,
        program: RunUnit,
        key: Ed25519PrivateKey,
        rpc: RpcClient,
        proposer: AccountAddress,
    ) -> Self {
        Self {
            proposer_address: proposer,
            inner: SharedData {
                client: rpc,
                key,
                program,
                flexemu_config,
            },
        }
    }
    pub fn run(
        &self,
        // exec: Vec<u8>,
        // argv: Vec<String>,
        // envs: Vec<(String, String)>,
    ) -> anyhow::Result<()> {
        loop {
            let remote_reader = self.inner.client.state_reader(StateRootOption::Latest)?;
            let declared_state = remote_reader.get_resource::<Global>(self.proposer_address)?;
            if let Some(Global { declared_state }) = declared_state {
                let emu_state = run_mips(
                    self.inner.flexemu_config.clone(),
                    self.inner.program.binary.clone(),
                    self.inner.program.argv.clone(),
                    self.inner.program.env.clone(),
                    None,
                )?;
                let my_state_root = HashValue::new(emu_state.state_root());
                if my_state_root == declared_state {
                    info!("state check ok, quit now");
                    break;
                }

                let remote_reader = self.inner.client.state_reader(StateRootOption::Latest)?;
                let challenges = remote_reader
                    .get_resource::<Challenges>(self.proposer_address)?
                    .unwrap();

                // if no challenge exists, fire a challenge
                if challenges
                    .value
                    .iter()
                    .find(|c| c.challenger == self.inner.self_address() && c.success == 0)
                    .is_none()
                {
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
                }

                let remote_reader = self.inner.client.state_reader(StateRootOption::Latest)?;
                let challenges = remote_reader
                    .get_resource::<Challenges>(self.proposer_address)?
                    .unwrap();
                for (id, c) in challenges.value.into_iter().enumerate() {
                    if c.challenger == self.inner.self_address() && c.success == 0 {
                        self.handle_challenge(id as u64, c)?;
                    }
                }
                // sleep 3s
                sleep(Duration::from_secs(3));
            } else {
                info!("waiting for proposer's proposal data");
            }
            sleep(Duration::from_secs(3));
        }
        Ok(())
    }
    fn handle_challenge(&self, cid: u64, c: ChallengeData) -> anyhow::Result<()> {
        let proposer_address = self.proposer_address;
        if c.l + 1 == c.r {
            let state_change = run_mips_state_change(
                self.inner.flexemu_config.clone(),
                self.inner.program.binary.clone(),
                self.inner.program.argv.clone(),
                self.inner.program.env.clone(),
                c.l as usize,
            )?;
            debug_assert!(c.r == state_change.step);
            info!(
                "prepare to deny_state_transition by run step {} -> {}",
                c.l, c.r
            );
            let state_proof = generate_step_proof(state_change);
            let txn = self.inner.build_txn(deny_state_transition(
                proposer_address,
                cid,
                state_proof.access_nodes,
            ))?;
            let txn_info = self.inner.send_and_wait_txn(txn)?;
            if txn_info.status == TransactionStatusView::Executed {
                info!(
                    "deny_state_transition of {}-{} from  {} -> {}, with root {} -> {}",
                    proposer_address,
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
                    self.inner.flexemu_config.clone(),
                    self.inner.program.binary.clone(),
                    self.inner.program.argv.clone(),
                    self.inner.program.env.clone(),
                    Some(next_step as usize),
                )?;
                let state_root = HashValue::new(state.state_root());
                let txn = self
                    .inner
                    .build_txn(assert_state(proposer_address, cid, state_root))?;
                let _txn_info = self.inner.send_and_wait_txn(txn)?;
                info!("assert state {:?} at step {}", state_root, next_step);
            }
        }
        Ok(())
    }
}

fn run_mips(
    config: FlexEmuConfig,
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
    config: FlexEmuConfig,
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
