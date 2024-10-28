use serde::{Deserialize, Deserializer, Serializer};
use thiserror::Error;

/// Parse error code.
pub const PARSE_ERROR_CODE: i32 = -32700;
/// Invalid request error code.
pub const INVALID_REQUEST_CODE: i32 = -32600;
/// Method not found error code.
pub const METHOD_NOT_FOUND_CODE: i32 = -32601;
/// Invalid params error code.
pub const INVALID_PARAMS_CODE: i32 = -32602;
/// Internal error code.
pub const INTERNAL_ERROR_CODE: i32 = -32603;
/// Custom server error when a call failed.
pub const CALL_EXECUTION_FAILED_CODE: i32 = -32000;
/// Unknown error.
pub const UNKNOWN_ERROR_CODE: i32 = -32001;
/// Batched requests are not supported by the server.
pub const BATCHES_NOT_SUPPORTED_CODE: i32 = -32005;
/// Subscription limit per connection was exceeded.
pub const TOO_MANY_SUBSCRIPTIONS_CODE: i32 = -32006;
/// Oversized request error code.
pub const OVERSIZED_REQUEST_CODE: i32 = -32007;
/// Oversized response error code.
pub const OVERSIZED_RESPONSE_CODE: i32 = -32008;
/// Server is busy error code.
pub const SERVER_IS_BUSY_CODE: i32 = -32009;
/// Batch request limit was exceed.
pub const TOO_BIG_BATCH_REQUEST_CODE: i32 = -32010;
/// Batch request limit was exceed.
pub const TOO_BIG_BATCH_RESPONSE_CODE: i32 = -32011;

/// Parse error message
pub const PARSE_ERROR_MSG: &str = "Parse error";
/// Oversized request message
pub const OVERSIZED_REQUEST_MSG: &str = "Request is too big";
/// Oversized response message
pub const OVERSIZED_RESPONSE_MSG: &str = "Response is too big";
/// Internal error message.
pub const INTERNAL_ERROR_MSG: &str = "Internal error";
/// Invalid params error message.
pub const INVALID_PARAMS_MSG: &str = "Invalid params";
/// Invalid params error message.
pub const NO_PARAMS_EXPECTED_MSG: &str = "Invalid parameters: No parameters were expected";
/// Invalid request error message.
pub const INVALID_REQUEST_MSG: &str = "Invalid request";
/// Method not found error message.
pub const METHOD_NOT_FOUND_MSG: &str = "Method not found";
/// Server is busy error message.
pub const SERVER_IS_BUSY_MSG: &str = "Server is busy, try again later";
/// Reserved for implementation-defined server-errors.
pub const SERVER_ERROR_MSG: &str = "Server error";
/// Batched requests not supported error message.
pub const BATCHES_NOT_SUPPORTED_MSG: &str = "Batched requests are not supported by this server";
/// Subscription limit per connection was exceeded.
pub const TOO_MANY_SUBSCRIPTIONS_MSG: &str = "Too many subscriptions on the connection";
/// Batched requests limit was exceed.
pub const TOO_BIG_BATCH_REQUEST_MSG: &str = "The batch request was too large";
/// Batch request response limit was exceed.
pub const TOO_BIG_BATCH_RESPONSE_MSG: &str = "The batch response was too large";

/// JSONRPC error code
#[derive(Error, Debug, PartialEq, Eq, Copy, Clone)]
pub enum ErrorCode {
    /// Invalid JSON was received by the server.
    /// An error occurred on the server while parsing the JSON text.
    ParseError,
    /// The request was too big.
    OversizedRequest,
    /// The JSON sent is not a valid Request object.
    InvalidRequest,
    /// The method does not exist / is not available.
    MethodNotFound,
    /// Server is busy / resources are at capacity.
    ServerIsBusy,
    /// Invalid method parameter(s).
    InvalidParams,
    /// Internal JSON-RPC error.
    InternalError,
    /// Reserved for implementation-defined server-errors.
    ServerError(i32),
}

impl ErrorCode {
    /// Returns integer code value
    pub const fn code(&self) -> i32 {
        use ErrorCode::*;
        match *self {
            ParseError => PARSE_ERROR_CODE,
            OversizedRequest => OVERSIZED_REQUEST_CODE,
            InvalidRequest => INVALID_REQUEST_CODE,
            MethodNotFound => METHOD_NOT_FOUND_CODE,
            ServerIsBusy => SERVER_IS_BUSY_CODE,
            InvalidParams => INVALID_PARAMS_CODE,
            InternalError => INTERNAL_ERROR_CODE,
            ServerError(code) => code,
        }
    }

    /// Returns the message for the given error code.
    pub const fn message(&self) -> &'static str {
        use ErrorCode::*;
        match self {
            ParseError => PARSE_ERROR_MSG,
            OversizedRequest => OVERSIZED_REQUEST_MSG,
            InvalidRequest => INVALID_REQUEST_MSG,
            MethodNotFound => METHOD_NOT_FOUND_MSG,
            ServerIsBusy => SERVER_IS_BUSY_MSG,
            InvalidParams => INVALID_PARAMS_MSG,
            InternalError => INTERNAL_ERROR_MSG,
            ServerError(_) => SERVER_ERROR_MSG,
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code(), self.message())
    }
}

impl From<i32> for ErrorCode {
    fn from(code: i32) -> Self {
        use ErrorCode::*;
        match code {
            PARSE_ERROR_CODE => ParseError,
            OVERSIZED_REQUEST_CODE => OversizedRequest,
            INVALID_REQUEST_CODE => InvalidRequest,
            METHOD_NOT_FOUND_CODE => MethodNotFound,
            INVALID_PARAMS_CODE => InvalidParams,
            INTERNAL_ERROR_CODE => InternalError,
            code => ServerError(code),
        }
    }
}

impl<'a> serde::Deserialize<'a> for ErrorCode {
    fn deserialize<D>(deserializer: D) -> Result<ErrorCode, D::Error>
    where
        D: Deserializer<'a>,
    {
        let code: i32 = Deserialize::deserialize(deserializer)?;
        Ok(ErrorCode::from(code))
    }
}

impl serde::Serialize for ErrorCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(self.code())
    }
}
