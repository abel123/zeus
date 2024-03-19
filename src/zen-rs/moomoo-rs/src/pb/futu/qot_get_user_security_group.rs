#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct C2s {
    /// GroupType,自选股分组类型。
    #[prost(int32, required, tag = "1")]
    pub group_type: i32,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GroupData {
    /// 自选股分组名字
    #[prost(string, required, tag = "1")]
    pub group_name: ::prost::alloc::string::String,
    /// GroupType,自选股分组类型。
    #[prost(int32, required, tag = "2")]
    pub group_type: i32,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct S2c {
    /// 自选股分组列表
    #[prost(message, repeated, tag = "1")]
    pub group_list: ::prost::alloc::vec::Vec<GroupData>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Request {
    #[prost(message, required, tag = "1")]
    pub c2s: C2s,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Response {
    /// RetType,返回结果
    #[prost(int32, required, tag = "1", default = "-400")]
    pub ret_type: i32,
    #[prost(string, optional, tag = "2")]
    pub ret_msg: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(int32, optional, tag = "3")]
    pub err_code: ::core::option::Option<i32>,
    #[prost(message, optional, tag = "4")]
    pub s2c: ::core::option::Option<S2c>,
}
/// 自选股分组类型
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum GroupType {
    /// 未知
    Unknown = 0,
    /// 自定义分组
    Custom = 1,
    /// 系统分组
    System = 2,
    /// 全部分组
    All = 3,
}
impl GroupType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            GroupType::Unknown => "GroupType_Unknown",
            GroupType::Custom => "GroupType_Custom",
            GroupType::System => "GroupType_System",
            GroupType::All => "GroupType_All",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "GroupType_Unknown" => Some(Self::Unknown),
            "GroupType_Custom" => Some(Self::Custom),
            "GroupType_System" => Some(Self::System),
            "GroupType_All" => Some(Self::All),
            _ => None,
        }
    }
}
