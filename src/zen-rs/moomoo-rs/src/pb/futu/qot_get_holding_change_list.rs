#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct C2s {
    /// 股票
    #[prost(message, required, tag = "1")]
    pub security: super::qot_common::Security,
    /// 持有者类别（1机构、2基金、3高管）
    #[prost(int32, required, tag = "2")]
    pub holder_category: i32,
    /// 以下是发布时间筛选，不传返回所有数据，传了返回发布时间属于开始时间到结束时间段内的数据
    ///
    /// 开始时间，严格按YYYY-MM-DD HH:MM:SS或YYYY-MM-DD HH:MM:SS.MS格式传
    #[prost(string, optional, tag = "3")]
    pub begin_time: ::core::option::Option<::prost::alloc::string::String>,
    /// 结束时间，严格按YYYY-MM-DD HH:MM:SS或YYYY-MM-DD HH:MM:SS.MS格式传
    #[prost(string, optional, tag = "4")]
    pub end_time: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct S2c {
    /// 股票
    #[prost(message, required, tag = "1")]
    pub security: super::qot_common::Security,
    /// 对应类别的持股变化列表（最多返回前100大股东的变化）
    #[prost(message, repeated, tag = "2")]
    pub holding_change_list: ::prost::alloc::vec::Vec<
        super::qot_common::ShareHoldingChange,
    >,
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
