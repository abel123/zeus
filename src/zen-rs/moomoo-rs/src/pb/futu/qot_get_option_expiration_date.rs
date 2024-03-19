#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct C2s {
    /// 期权标的股，目前仅支持传入港美正股以及恒指国指
    #[prost(message, required, tag = "1")]
    pub owner: super::qot_common::Security,
    /// Qot_Common.IndexOptionType，指数期权的类型，仅用于恒指国指
    #[prost(int32, optional, tag = "2")]
    pub index_option_type: ::core::option::Option<i32>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OptionExpirationDate {
    /// 期权链行权日（港股和 A 股市场默认是北京时间，美股市场默认是美东时间）
    #[prost(string, optional, tag = "1")]
    pub strike_time: ::core::option::Option<::prost::alloc::string::String>,
    /// 行权日时间戳
    #[prost(double, optional, tag = "2")]
    pub strike_timestamp: ::core::option::Option<f64>,
    /// 距离到期日天数，负数表示已过期
    #[prost(int32, required, tag = "3")]
    pub option_expiry_date_distance: i32,
    /// Qot_Common.ExpirationCycle,交割周期（仅用于香港指数期权）
    #[prost(int32, optional, tag = "4")]
    pub cycle: ::core::option::Option<i32>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct S2c {
    /// 期权链行权日
    #[prost(message, repeated, tag = "1")]
    pub date_list: ::prost::alloc::vec::Vec<OptionExpirationDate>,
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
