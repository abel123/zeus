#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct C2s {
    /// 是否返回所有连接的订阅状态,不传或者传false只返回当前连接数据
    #[prost(bool, optional, tag = "1")]
    pub is_req_all_conn: ::core::option::Option<bool>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct S2c {
    /// 订阅订阅信息
    #[prost(message, repeated, tag = "1")]
    pub conn_sub_info_list: ::prost::alloc::vec::Vec<super::qot_common::ConnSubInfo>,
    /// FutuOpenD已使用的订阅额度
    #[prost(int32, required, tag = "2")]
    pub total_used_quota: i32,
    /// FutuOpenD剩余订阅额度
    #[prost(int32, required, tag = "3")]
    pub remain_quota: i32,
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
