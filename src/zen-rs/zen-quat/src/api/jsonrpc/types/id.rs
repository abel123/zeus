use serde::{Deserialize, Serialize};

/// Request Id
#[derive(Debug, PartialEq, Clone, Hash, Eq, Deserialize, Serialize, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum Id {
    /// Null
    Null,
    /// Numeric id
    Number(u64),
    /// String id
    Str(String),
}
