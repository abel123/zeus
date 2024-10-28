use actix::Message;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::error::code::ErrorCode;
use super::JSONRPC_VERSION;
use super::{error::object::ErrorObject, id::Id};

/// When a rpc call is made, the Server MUST reply with a Response, except for in the case of Notifications. The
/// Response is expressed as a single JSON Object, with the following members:
#[derive(Debug, Serialize, Deserialize, Message)]
#[rtype(result = "()")]
pub struct RpcResponse {
    /// A String specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
    pub jsonrpc: String,
    /// Payload which can be result or error.
    #[serde(flatten)]
    pub payload: RpcPayload,
    /// This member is REQUIRED.
    /// It MUST be the same as the value of the id member in the Request Object.
    /// If there was an error in detecting the id in the Request object (e.g. Parse error/Invalid Request),
    /// it MUST be Null.
    pub id: Id,
}

impl RpcResponse {
    /// Prints out the value as JSON string.
    pub fn dump(&self) -> String {
        serde_json::to_string(self).expect("Should never failed")
    }
}

/// Represent the payload of the JSON-RPC response object
///
/// It can be:
///
/// ```json
/// "result":<value>
/// "error":{"code":<code>,"message":<msg>,"data":<data>}
/// ```
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RpcPayload {
    /// Corresponds to successful JSON-RPC response with the field `result`.
    Result(Value),
    /// Corresponds to failed JSON-RPC response with a error object with the field `error.
    Error(ErrorObject),
}

impl Default for RpcPayload {
    fn default() -> Self {
        Self::Result(Value::Null)
    }
}

impl From<ErrorCode> for RpcPayload {
    fn from(code: ErrorCode) -> Self {
        Self::Error(code.into())
    }
}

impl From<ErrorObject> for RpcPayload {
    fn from(obj: ErrorObject) -> Self {
        Self::Error(obj)
    }
}

impl From<Value> for RpcPayload {
    fn from(value: Value) -> Self {
        Self::Result(value)
    }
}

impl Default for RpcResponse {
    fn default() -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.into(),
            payload: RpcPayload::default(),
            id: Id::Null,
        }
    }
}
