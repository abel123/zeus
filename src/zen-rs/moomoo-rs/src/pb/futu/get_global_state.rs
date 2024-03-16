#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct C2s {
    /// 历史原因，目前已废弃，填0即可
    #[prost(uint64, required, tag = "1")]
    pub user_id: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct S2c {
    /// Qot_Common.QotMarketState,港股主板市场状态
    #[prost(int32, required, tag = "1")]
    pub market_hk: i32,
    /// Qot_Common.QotMarketState,美股Nasdaq市场状态
    #[prost(int32, required, tag = "2")]
    pub market_us: i32,
    /// Qot_Common.QotMarketState,沪市状态
    #[prost(int32, required, tag = "3")]
    pub market_sh: i32,
    /// Qot_Common.QotMarketState,深市状态
    #[prost(int32, required, tag = "4")]
    pub market_sz: i32,
    /// Qot_Common.QotMarketState,港股期货市场状态
    #[prost(int32, required, tag = "5")]
    pub market_hk_future: i32,
    /// Qot_Common.QotMarketState,美国期货市场状态
    #[prost(int32, optional, tag = "15")]
    pub market_us_future: ::core::option::Option<i32>,
    /// Qot_Common.QotMarketState,新加坡期货市场状态
    #[prost(int32, optional, tag = "17")]
    pub market_sg_future: ::core::option::Option<i32>,
    /// Qot_Common.QotMarketState,日本期货市场状态
    #[prost(int32, optional, tag = "18")]
    pub market_jp_future: ::core::option::Option<i32>,
    /// 是否登陆行情服务器
    #[prost(bool, required, tag = "6")]
    pub qot_logined: bool,
    /// 是否登陆交易服务器
    #[prost(bool, required, tag = "7")]
    pub trd_logined: bool,
    /// 版本号
    #[prost(int32, required, tag = "8")]
    pub server_ver: i32,
    /// buildNo
    #[prost(int32, required, tag = "9")]
    pub server_build_no: i32,
    /// 当前服务器时间
    #[prost(int64, required, tag = "10")]
    pub time: i64,
    /// 当前本地时间
    #[prost(double, optional, tag = "11")]
    pub local_time: ::core::option::Option<f64>,
    /// 当前程序状态
    #[prost(message, optional, tag = "12")]
    pub program_status: ::core::option::Option<super::common::ProgramStatus>,
    #[prost(string, optional, tag = "13")]
    pub qot_svr_ip_addr: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "14")]
    pub trd_svr_ip_addr: ::core::option::Option<::prost::alloc::string::String>,
    /// 此连接的连接ID，连接的唯一标识
    #[prost(uint64, optional, tag = "16")]
    pub conn_id: ::core::option::Option<u64>,
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
