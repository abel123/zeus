#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct C2s {
    /// 历史原因，目前已废弃，填0即可
    #[prost(uint64, required, tag = "1")]
    pub user_id: u64,
    /// 交易品类，参考 Trd_Common.TrdCategory
    #[prost(int32, optional, tag = "2")]
    pub trd_category: ::core::option::Option<i32>,
    /// 是否返回全能账户，仅SG用户需要
    #[prost(bool, optional, tag = "3")]
    pub need_general_sec_account: ::core::option::Option<bool>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct S2c {
    /// 交易业务账户列表
    #[prost(message, repeated, tag = "1")]
    pub acc_list: ::prost::alloc::vec::Vec<super::trd_common::TrdAcc>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Request {
    #[prost(message, required, tag = "1")]
    pub c2s: C2s,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Response {
    /// 以下3个字段每条协议都有，注释说明在InitConnect.proto中
    #[prost(int32, required, tag = "1", default = "-400")]
    pub ret_type: i32,
    #[prost(string, optional, tag = "2")]
    pub ret_msg: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(int32, optional, tag = "3")]
    pub err_code: ::core::option::Option<i32>,
    #[prost(message, optional, tag = "4")]
    pub s2c: ::core::option::Option<S2c>,
}
