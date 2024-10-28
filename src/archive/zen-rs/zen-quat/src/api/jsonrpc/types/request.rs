use serde::{Deserialize, Serialize};

use super::{id::Id, params::Params};

/// A rpc call is represented by sending a Request object to a Server.
#[derive(Debug, Serialize, Deserialize)]
pub struct RpcRequest {
    /// A String specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
    pub jsonrpc: String,

    /// A String containing the name of the method to be invoked. Method names that begin with the word rpc followed by
    /// a period character (U+002E or ASCII 46) are reserved for rpc-internal methods and extensions and MUST NOT be
    /// used for anything else.
    pub method: String,

    /// A Structured value that holds the parameter values to be used during the invocation of the method. This member
    /// MAY be omitted.
    pub params: Params,

    /// An identifier established by the Client that MUST contain a String, Number, or NULL value if included. If it is
    /// not included it is assumed to be a notification. The value SHOULD normally not be Null [1] and Numbers SHOULD
    /// NOT contain fractional parts.
    pub id: Id,
}

impl RpcRequest {
    /// Prints out the value as JSON string.
    pub fn dump(&self) -> String {
        serde_json::to_string(self).expect("Should never failed")
    }
}
