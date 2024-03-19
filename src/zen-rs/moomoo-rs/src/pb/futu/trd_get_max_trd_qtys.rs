#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct C2s {
    /// 交易公共参数头
    #[prost(message, required, tag = "1")]
    pub header: super::trd_common::TrdHeader,
    /// 订单类型, 参见Trd_Common.OrderType的枚举定义
    #[prost(int32, required, tag = "2")]
    pub order_type: i32,
    /// 代码，港股必须是5位数字，A股必须是6位数字，美股没限制
    #[prost(string, required, tag = "3")]
    pub code: ::prost::alloc::string::String,
    /// 价格，（证券账户精确到小数点后 3 位，期货账户精确到小数点后 9 位，超出部分会被舍弃）。如果是竞价、市价单，请也填入一个当前价格，服务器才好计算
    #[prost(double, required, tag = "4")]
    pub price: f64,
    /// 订单号，新下订单不需要，如果是修改订单就需要把原订单号带上才行，因为改单的最大买卖数量会包含原订单数量。
    #[prost(uint64, optional, tag = "5")]
    pub order_id: ::core::option::Option<u64>,
    /// 为保证与下单的价格同步，也提供调整价格选项，以下2个为调整价格使用，对港、A股有意义，因为港股有价位，A股2位精度，美股可不传
    ///
    /// 是否调整价格，如果价格不合法，是否调整到合法价位，true调整，false不调整
    #[prost(bool, optional, tag = "6")]
    pub adjust_price: ::core::option::Option<bool>,
    /// 调整方向和调整幅度百分比限制，正数代表向上调整，负数代表向下调整，具体值代表调整幅度限制，如：0.015代表向上调整且幅度不超过1.5%；-0.01代表向下调整且幅度不超过1%
    #[prost(double, optional, tag = "7")]
    pub adjust_side_and_limit: ::core::option::Option<f64>,
    /// 证券所属市场，参见TrdSecMarket的枚举定义
    #[prost(int32, optional, tag = "8")]
    pub sec_market: ::core::option::Option<i32>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct S2c {
    /// 交易公共参数头
    #[prost(message, required, tag = "1")]
    pub header: super::trd_common::TrdHeader,
    /// 最大可交易数量结构
    #[prost(message, optional, tag = "2")]
    pub max_trd_qtys: ::core::option::Option<super::trd_common::MaxTrdQtys>,
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
