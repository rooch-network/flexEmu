use serde::Deserialize;
use serde::Serialize;

use crate::loader::Config;

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
pub struct OmoConfig {
    pub os: Config,
}
