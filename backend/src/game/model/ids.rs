use serde::{Deserialize, Serialize};

pub type TerritoryId = usize;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct PlayerId(pub u8);
