#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct C2s {
    /// 交易写操作防重放攻击
    #[prost(message, required, tag = "1")]
    pub packet_id: super::common::PacketId,
    /// 交易公共参数头
    #[prost(message, required, tag = "2")]
    pub header: super::trd_common::TrdHeader,
    /// 订单号
    #[prost(uint64, required, tag = "3")]
    pub order_id: u64,
    /// 需要再次确认订单的原因，参见Trd_Common.ReconfirmOrderReason的枚举定义
    #[prost(int32, required, tag = "4")]
    pub reconfirm_reason: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct S2c {
    /// 交易公共参数头
    #[prost(message, required, tag = "1")]
    pub header: super::trd_common::TrdHeader,
    /// 订单号
    #[prost(uint64, required, tag = "2")]
    pub order_id: u64,
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
