#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct C2s {
    /// 股票列表
    #[prost(message, repeated, tag = "1")]
    pub security_list: ::prost::alloc::vec::Vec<super::qot_common::Security>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MarketInfo {
    /// 股票代码
    #[prost(message, required, tag = "1")]
    pub security: super::qot_common::Security,
    /// 股票名称
    #[prost(string, required, tag = "2")]
    pub name: ::prost::alloc::string::String,
    /// Qot_Common.QotMarketState,市场状态
    #[prost(int32, required, tag = "3")]
    pub market_state: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct S2c {
    /// 市场状态信息
    #[prost(message, repeated, tag = "1")]
    pub market_info_list: ::prost::alloc::vec::Vec<MarketInfo>,
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
