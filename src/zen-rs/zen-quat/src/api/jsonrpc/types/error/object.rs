use super::code::ErrorCode;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;

/// [Failed JSON-RPC response object](https://www.jsonrpc.org/specification#response_object).
#[derive(Debug, Deserialize, Serialize, Clone, thiserror::Error)]
#[serde(deny_unknown_fields)]
#[error("{self:?}")]
pub struct ErrorObject {
    /// Code
    code: ErrorCode,
    /// Message
    message: String,
}

impl ErrorObject {
    pub fn new(code: impl Into<ErrorCode>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Return the error code
    pub fn code(&self) -> i32 {
        self.code.code()
    }

    /// Return the message
    pub fn message(&self) -> &str {
        self.message.borrow()
    }
}

impl From<ErrorCode> for ErrorObject {
    fn from(code: ErrorCode) -> Self {
        Self {
            code,
            message: code.message().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ErrorCode, ErrorObject};

    #[test]
    fn deserialize_works() {
        let ser = r#"{"code":-32700,"message":"Parse error"}"#;
        let exp: ErrorObject = ErrorCode::ParseError.into();
        let err: ErrorObject = serde_json::from_str(ser).unwrap();
        assert_eq!(exp, err);
    }

    #[test]
    fn deserialize_with_optional_data() {
        let ser = r#"{"code":-32700,"message":"Parse error", "data":"vegan"}"#;
        let data = serde_json::value::to_raw_value(&"vegan").unwrap();
        let exp = ErrorObject::owned(ErrorCode::ParseError.code(), "Parse error", Some(data));
        let err: ErrorObject = serde_json::from_str(ser).unwrap();
        assert_eq!(exp, err);
    }

    #[test]
    fn deserialized_error_with_quoted_str() {
        let raw = r#"{
				"code": 1002,
				"message": "desc: \"Could not decode `ChargeAssetTxPayment::asset_id`\" } })",
				"data": "\\\"validate_transaction\\\""
		}"#;
        let err: ErrorObject = serde_json::from_str(raw).unwrap();

        let data = serde_json::value::to_raw_value(&"\\\"validate_transaction\\\"").unwrap();
        let exp = ErrorObject::borrowed(
            1002,
            &"desc: \"Could not decode `ChargeAssetTxPayment::asset_id`\" } })",
            Some(&*data),
        );

        assert_eq!(err, exp);
    }

    #[test]
    fn serialize_works() {
        let exp = r#"{"code":-32603,"message":"Internal error"}"#;
        let err: ErrorObject = ErrorCode::InternalError.into();
        let ser = serde_json::to_string(&err).unwrap();
        assert_eq!(exp, ser);
    }

    #[test]
    fn serialize_optional_data_works() {
        let exp = r#"{"code":-32699,"message":"food","data":"not vegan"}"#;
        let data = serde_json::value::to_raw_value(&"not vegan").unwrap();
        let ser = serde_json::to_string(&ErrorObject::owned(-32699, "food", Some(data))).unwrap();
        assert_eq!(exp, ser);
    }
}
