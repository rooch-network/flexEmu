use crate::txn_builder::CHALLENGE_ADDRESS;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_types::{account_address::AccountAddress, language_storage::StructTag};
use starcoin_vm_types::move_resource::MoveResource;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeData {
    pub l: u64,
    pub r: u64,
    pub asserted_state: MoveTable,
    pub defended_state: MoveTable,
    pub challenger: AccountAddress,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Global {
    pub declared_state: HashValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenges {
    pub value: Vec<ChallengeData>,
}
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct MoveTable {
    pub handle: u128,
    pub length: u64,
}

impl MoveResource for ChallengeData {
    const MODULE_NAME: &'static str = "SimpleChallenge";
    const STRUCT_NAME: &'static str = "ChallengeData";
    fn struct_tag() -> StructTag {
        StructTag {
            address: CHALLENGE_ADDRESS,
            module: Self::module_identifier(),
            name: Self::struct_identifier(),
            type_params: Self::type_params(),
        }
    }
}

impl MoveResource for Challenges {
    const MODULE_NAME: &'static str = "SimpleChallenge";
    const STRUCT_NAME: &'static str = "Challenges";
    fn struct_tag() -> StructTag {
        StructTag {
            address: CHALLENGE_ADDRESS,
            module: Self::module_identifier(),
            name: Self::struct_identifier(),
            type_params: Self::type_params(),
        }
    }
}
impl MoveResource for Global {
    const MODULE_NAME: &'static str = "SimpleChallenge";
    const STRUCT_NAME: &'static str = "Global";
    fn struct_tag() -> StructTag {
        StructTag {
            address: CHALLENGE_ADDRESS,
            module: Self::module_identifier(),
            name: Self::struct_identifier(),
            type_params: Self::type_params(),
        }
    }
}
