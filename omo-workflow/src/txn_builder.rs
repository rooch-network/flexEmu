use clap::Id;
use starcoin_crypto::HashValue;
use starcoin_rpc_api::types::ContractCall;
use starcoin_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::ModuleId,
    transaction::{RawUserTransaction, ScriptFunction, TransactionArgument, TransactionPayload},
};
use starcoin_vm_types::language_storage::FunctionId;
use std::str::FromStr;

// TODO: change me
pub const CHALLENGE_ADDRESS: AccountAddress = AccountAddress::from_str("0x1").unwrap();

pub const CHALLENGE_MODULE: ModuleId = ModuleId::new(
    CHALLENGE_ADDRESS,
    Identifier::from_str("challenge_script").unwrap(),
);
pub fn declare_state(final_state: HashValue) -> TransactionPayload {
    TransactionPayload::ScriptFunction(ScriptFunction::new(
        CHALLENGE_MODULE.clone(),
        Identifier::from_str("declare_state").unwrap(),
        vec![],
        vec![bcs_ext::to_bytes(&final_state.to_vec()).unwrap()],
    ))
}

pub fn create_challenge(
    proposer_address: AccountAddress,
    final_system_state: HashValue,
    step_count: u64,
) -> TransactionPayload {
    TransactionPayload::ScriptFunction(ScriptFunction::new(
        CHALLENGE_MODULE.clone(),
        Identifier::from_str("create_challenge").unwrap(),
        vec![],
        vec![
            bcs_ext::to_bytes(&proposer_address).unwrap(),
            bcs_ext::to_bytes(&final_system_state.to_vec()).unwrap(),
            bcs_ext::to_bytes(&step_count).unwrap(),
        ],
    ))
}

pub fn assert_state(
    proposer: AccountAddress,
    challenge_id: u64,
    state: HashValue,
) -> TransactionPayload {
    TransactionPayload::ScriptFunction(ScriptFunction::new(
        CHALLENGE_MODULE.clone(),
        Identifier::from_str("assert_state").unwrap(),
        vec![],
        vec![
            bcs_ext::to_bytes(&proposer).unwrap(),
            bcs_ext::to_bytes(&challenge_id).unwrap(),
            bcs_ext::to_bytes(state.as_slice()).unwrap(),
        ],
    ))
}

pub fn defend_state(challenge_id: u64, state: HashValue) -> TransactionPayload {
    TransactionPayload::ScriptFunction(ScriptFunction::new(
        CHALLENGE_MODULE.clone(),
        Identifier::from_str("defend_state").unwrap(),
        vec![],
        vec![
            bcs_ext::to_bytes(&challenge_id).unwrap(),
            bcs_ext::to_bytes(state.as_slice()).unwrap(),
        ],
    ))
}

pub fn confirm_state_transition(
    proposer: AccountAddress,
    challenge_id: u64,
    state_data: Vec<u8>,
) -> TransactionPayload {
    TransactionPayload::ScriptFunction(ScriptFunction::new(
        CHALLENGE_MODULE.clone(),
        Identifier::from_str("confirm_state_transition").unwrap(),
        vec![],
        vec![
            bcs_ext::to_bytes(&proposer).unwrap(),
            bcs_ext::to_bytes(&challenge_id).unwrap(),
            bcs_ext::to_bytes(&state_data).unwrap(),
        ],
    ))
}

pub fn deny_state_transition(
    proposer: AccountAddress,
    challenge_id: u64,
    state_data: Vec<u8>,
) -> TransactionPayload {
    TransactionPayload::ScriptFunction(ScriptFunction::new(
        CHALLENGE_MODULE.clone(),
        Identifier::from_str("deny_state_transition").unwrap(),
        vec![],
        vec![
            bcs_ext::to_bytes(&proposer).unwrap(),
            bcs_ext::to_bytes(&challenge_id).unwrap(),
            bcs_ext::to_bytes(&state_data).unwrap(),
        ],
    ))
}

pub fn contain_state(
    proposer: AccountAddress,
    challenge_id: u64,
    step: u64,
    defend: bool,
) -> ContractCall {
    ContractCall {
        function_id: FunctionId {
            module: CHALLENGE_MODULE,
            function: Identifier::from_str("contain_state").unwrap(),
        }
        .into(),
        type_args: vec![],
        args: vec![
            TransactionArgument::Address(proposer).into(),
            TransactionArgument::U64(challenge_id).into(),
            TransactionArgument::U64(step).into(),
            TransactionArgument::Bool(defend).into(),
        ],
    }
}
