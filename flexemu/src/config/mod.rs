use serde::{Deserialize, Serialize};

use crate::loader::Config;

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
pub struct FlexEmuConfig {
    pub os: Config,
}
