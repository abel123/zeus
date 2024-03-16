/// 以下为数据字段筛选，可选字段，不填表示不过滤
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DataFilter {
    /// 隐含波动率过滤起点（精确到小数点后 0 位，超出部分会被舍弃）
    #[prost(double, optional, tag = "1")]
    pub implied_volatility_min: ::core::option::Option<f64>,
    /// 隐含波动率过滤终点（精确到小数点后 0 位，超出部分会被舍弃）
    #[prost(double, optional, tag = "2")]
    pub implied_volatility_max: ::core::option::Option<f64>,
    /// 希腊值 Delta过滤起点（精确到小数点后 3 位，超出部分会被舍弃）
    #[prost(double, optional, tag = "3")]
    pub delta_min: ::core::option::Option<f64>,
    /// 希腊值 Delta过滤终点（精确到小数点后 3 位，超出部分会被舍弃）
    #[prost(double, optional, tag = "4")]
    pub delta_max: ::core::option::Option<f64>,
    /// 希腊值 Gamma过滤起点（精确到小数点后 3 位，超出部分会被舍弃）
    #[prost(double, optional, tag = "5")]
    pub gamma_min: ::core::option::Option<f64>,
    /// 希腊值 Gamma过滤终点（精确到小数点后 3 位，超出部分会被舍弃）
    #[prost(double, optional, tag = "6")]
    pub gamma_max: ::core::option::Option<f64>,
    /// 希腊值 Vega过滤起点（精确到小数点后 3 位，超出部分会被舍弃）
    #[prost(double, optional, tag = "7")]
    pub vega_min: ::core::option::Option<f64>,
    /// 希腊值 Vega过滤终点（精确到小数点后 3 位，超出部分会被舍弃）
    #[prost(double, optional, tag = "8")]
    pub vega_max: ::core::option::Option<f64>,
    /// 希腊值 Theta过滤起点（精确到小数点后 3 位，超出部分会被舍弃）
    #[prost(double, optional, tag = "9")]
    pub theta_min: ::core::option::Option<f64>,
    /// 希腊值 Theta过滤终点（精确到小数点后 3 位，超出部分会被舍弃）
    #[prost(double, optional, tag = "10")]
    pub theta_max: ::core::option::Option<f64>,
    /// 希腊值 Rho过滤起点（精确到小数点后 3 位，超出部分会被舍弃）
    #[prost(double, optional, tag = "11")]
    pub rho_min: ::core::option::Option<f64>,
    /// 希腊值 Rho过滤终点（精确到小数点后 3 位，超出部分会被舍弃）
    #[prost(double, optional, tag = "12")]
    pub rho_max: ::core::option::Option<f64>,
    /// 净未平仓合约数过滤起点（精确到小数点后 0 位，超出部分会被舍弃）
    #[prost(double, optional, tag = "13")]
    pub net_open_interest_min: ::core::option::Option<f64>,
    /// 净未平仓合约数过滤终点（精确到小数点后 0 位，超出部分会被舍弃）
    #[prost(double, optional, tag = "14")]
    pub net_open_interest_max: ::core::option::Option<f64>,
    /// 未平仓合约数过滤起点（精确到小数点后 0 位，超出部分会被舍弃）
    #[prost(double, optional, tag = "15")]
    pub open_interest_min: ::core::option::Option<f64>,
    /// 未平仓合约数过滤终点（精确到小数点后 0 位，超出部分会被舍弃）
    #[prost(double, optional, tag = "16")]
    pub open_interest_max: ::core::option::Option<f64>,
    /// 成交量过滤起点（精确到小数点后 0 位，超出部分会被舍弃）
    #[prost(double, optional, tag = "17")]
    pub vol_min: ::core::option::Option<f64>,
    /// 成交量过滤终点（精确到小数点后 0 位，超出部分会被舍弃）
    #[prost(double, optional, tag = "18")]
    pub vol_max: ::core::option::Option<f64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct C2s {
    /// 期权标的股，目前仅支持传入港美正股以及恒指国指
    #[prost(message, required, tag = "1")]
    pub owner: super::qot_common::Security,
    /// Qot_Common.IndexOptionType，指数期权的类型，仅用于恒指国指
    #[prost(int32, optional, tag = "6")]
    pub index_option_type: ::core::option::Option<i32>,
    /// Qot_Common.OptionType，期权类型，可选字段，不指定则表示都返回
    #[prost(int32, optional, tag = "2")]
    pub r#type: ::core::option::Option<i32>,
    /// OptionCondType，价内价外，可选字段，不指定则表示都返回
    #[prost(int32, optional, tag = "3")]
    pub condition: ::core::option::Option<i32>,
    /// 期权到期日开始时间
    #[prost(string, required, tag = "4")]
    pub begin_time: ::prost::alloc::string::String,
    /// 期权到期日结束时间，时间跨度最多一个月
    #[prost(string, required, tag = "5")]
    pub end_time: ::prost::alloc::string::String,
    /// 数据字段筛选
    #[prost(message, optional, tag = "7")]
    pub data_filter: ::core::option::Option<DataFilter>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OptionItem {
    /// 看涨期权，不一定有该字段，由请求条件决定
    #[prost(message, optional, tag = "1")]
    pub call: ::core::option::Option<super::qot_common::SecurityStaticInfo>,
    /// 看跌期权，不一定有该字段，由请求条件决定
    #[prost(message, optional, tag = "2")]
    pub put: ::core::option::Option<super::qot_common::SecurityStaticInfo>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OptionChain {
    /// 行权日
    #[prost(string, required, tag = "1")]
    pub strike_time: ::prost::alloc::string::String,
    /// 期权信息
    #[prost(message, repeated, tag = "2")]
    pub option: ::prost::alloc::vec::Vec<OptionItem>,
    /// 行权日时间戳
    #[prost(double, optional, tag = "3")]
    pub strike_timestamp: ::core::option::Option<f64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct S2c {
    /// 期权链
    #[prost(message, repeated, tag = "1")]
    pub option_chain: ::prost::alloc::vec::Vec<OptionChain>,
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
    /// RetType，返回结果
    #[prost(int32, required, tag = "1", default = "-400")]
    pub ret_type: i32,
    #[prost(string, optional, tag = "2")]
    pub ret_msg: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(int32, optional, tag = "3")]
    pub err_code: ::core::option::Option<i32>,
    #[prost(message, optional, tag = "4")]
    pub s2c: ::core::option::Option<S2c>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum OptionCondType {
    Unknow = 0,
    /// 价内
    WithIn = 1,
    /// 价外
    Outside = 2,
}
impl OptionCondType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            OptionCondType::Unknow => "OptionCondType_Unknow",
            OptionCondType::WithIn => "OptionCondType_WithIn",
            OptionCondType::Outside => "OptionCondType_Outside",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "OptionCondType_Unknow" => Some(Self::Unknow),
            "OptionCondType_WithIn" => Some(Self::WithIn),
            "OptionCondType_Outside" => Some(Self::Outside),
            _ => None,
        }
    }
}
