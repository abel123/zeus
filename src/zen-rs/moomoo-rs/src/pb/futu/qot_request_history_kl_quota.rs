#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DetailItem {
    /// 拉取的股票
    #[prost(message, required, tag = "1")]
    pub security: super::qot_common::Security,
    /// 股票名称
    #[prost(string, optional, tag = "4")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    /// 拉取的时间字符串
    #[prost(string, required, tag = "2")]
    pub request_time: ::prost::alloc::string::String,
    /// 拉取的时间戳
    #[prost(int64, optional, tag = "3")]
    pub request_time_stamp: ::core::option::Option<i64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct C2s {
    /// 是否返回详细拉取过的历史纪录
    #[prost(bool, optional, tag = "2")]
    pub b_get_detail: ::core::option::Option<bool>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct S2c {
    /// 已使用过的额度，即当前周期内已经下载过多少只股票。
    #[prost(int32, required, tag = "1")]
    pub used_quota: i32,
    /// 剩余额度
    #[prost(int32, required, tag = "2")]
    pub remain_quota: i32,
    /// 每只拉取过的股票的下载时间
    #[prost(message, repeated, tag = "3")]
    pub detail_list: ::prost::alloc::vec::Vec<DetailItem>,
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
