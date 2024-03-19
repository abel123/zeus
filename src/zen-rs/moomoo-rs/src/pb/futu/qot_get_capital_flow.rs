#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct C2s {
    /// 股票
    #[prost(message, required, tag = "1")]
    pub security: super::qot_common::Security,
    /// Qot_Common.PeriodType 周期类型
    #[prost(int32, optional, tag = "2")]
    pub period_type: ::core::option::Option<i32>,
    /// 开始时间（格式：yyyy-MM-dd），仅周期类型不为实时有效
    #[prost(string, optional, tag = "3")]
    pub begin_time: ::core::option::Option<::prost::alloc::string::String>,
    /// 结束时间（格式：yyyy-MM-dd），仅周期类型不为实时有效
    #[prost(string, optional, tag = "4")]
    pub end_time: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CapitalFlowItem {
    /// 整体净流入
    #[prost(double, required, tag = "1")]
    pub in_flow: f64,
    /// 开始时间字符串,以分钟为单位
    #[prost(string, optional, tag = "2")]
    pub time: ::core::option::Option<::prost::alloc::string::String>,
    /// 开始时间戳
    #[prost(double, optional, tag = "3")]
    pub timestamp: ::core::option::Option<f64>,
    /// 主力大单净流入，仅周期类型不为实时有效
    #[prost(double, optional, tag = "4")]
    pub main_in_flow: ::core::option::Option<f64>,
    /// 特大单净流入
    #[prost(double, optional, tag = "5")]
    pub super_in_flow: ::core::option::Option<f64>,
    /// 大单净流入
    #[prost(double, optional, tag = "6")]
    pub big_in_flow: ::core::option::Option<f64>,
    /// 中单净流入
    #[prost(double, optional, tag = "7")]
    pub mid_in_flow: ::core::option::Option<f64>,
    /// 小单净流入
    #[prost(double, optional, tag = "8")]
    pub sml_in_flow: ::core::option::Option<f64>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct S2c {
    /// 资金流向
    #[prost(message, repeated, tag = "1")]
    pub flow_item_list: ::prost::alloc::vec::Vec<CapitalFlowItem>,
    /// 数据最后有效时间字符串
    #[prost(string, optional, tag = "2")]
    pub last_valid_time: ::core::option::Option<::prost::alloc::string::String>,
    /// 数据最后有效时间戳
    #[prost(double, optional, tag = "3")]
    pub last_valid_timestamp: ::core::option::Option<f64>,
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
