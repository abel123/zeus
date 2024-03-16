#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct C2s {
    /// 统计数据类型，DelayStatisticsType
    #[prost(int32, repeated, packed = "false", tag = "1")]
    pub type_list: ::prost::alloc::vec::Vec<i32>,
    /// 行情推送统计的区间，行情推送统计时有效，QotPushStage
    #[prost(int32, optional, tag = "2")]
    pub qot_push_stage: ::core::option::Option<i32>,
    /// 统计分段，默认100ms以下以2ms分段，100ms以上以500，1000，2000，-1分段，-1表示无穷大。
    #[prost(int32, repeated, packed = "false", tag = "3")]
    pub segment_list: ::prost::alloc::vec::Vec<i32>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelayStatisticsItem {
    /// 范围左闭右开，[begin,end)
    ///
    /// 耗时范围起点，毫秒单位
    #[prost(int32, required, tag = "1")]
    pub begin: i32,
    /// 耗时范围结束，毫秒单位
    #[prost(int32, required, tag = "2")]
    pub end: i32,
    /// 个数
    #[prost(int32, required, tag = "3")]
    pub count: i32,
    /// 占比, %
    #[prost(float, required, tag = "4")]
    pub proportion: f32,
    /// 累计占比, %
    #[prost(float, required, tag = "5")]
    pub cumulative_ratio: f32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelayStatistics {
    /// 行情推送类型,QotPushType
    #[prost(int32, required, tag = "1")]
    pub qot_push_type: i32,
    /// 统计信息
    #[prost(message, repeated, tag = "2")]
    pub item_list: ::prost::alloc::vec::Vec<DelayStatisticsItem>,
    /// 平均延迟
    #[prost(float, required, tag = "3")]
    pub delay_avg: f32,
    /// 总包数
    #[prost(int32, required, tag = "4")]
    pub count: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReqReplyStatisticsItem {
    /// 协议ID
    #[prost(int32, required, tag = "1")]
    pub proto_id: i32,
    /// 请求个数
    #[prost(int32, required, tag = "2")]
    pub count: i32,
    /// 平均总耗时，毫秒单位
    #[prost(float, required, tag = "3")]
    pub total_cost_avg: f32,
    /// 平均OpenD耗时，毫秒单位
    #[prost(float, required, tag = "4")]
    pub open_d_cost_avg: f32,
    /// 平均网络耗时，非当时实际请求网络耗时，毫秒单位
    #[prost(float, required, tag = "5")]
    pub net_delay_avg: f32,
    /// 是否本地直接回包，没有向服务器请求数据
    #[prost(bool, required, tag = "6")]
    pub is_local_reply: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlaceOrderStatisticsItem {
    /// 订单ID
    #[prost(string, required, tag = "1")]
    pub order_id: ::prost::alloc::string::String,
    /// 总耗时，毫秒单位
    #[prost(float, required, tag = "2")]
    pub total_cost: f32,
    /// OpenD耗时，毫秒单位
    #[prost(float, required, tag = "3")]
    pub open_d_cost: f32,
    /// 网络耗时，非当时实际请求网络耗时，毫秒单位
    #[prost(float, required, tag = "4")]
    pub net_delay: f32,
    /// 订单回包后到接收到订单下到交易所的耗时，毫秒单位
    #[prost(float, required, tag = "5")]
    pub update_cost: f32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct S2c {
    /// 行情推送延迟统计
    #[prost(message, repeated, tag = "1")]
    pub qot_push_statistics_list: ::prost::alloc::vec::Vec<DelayStatistics>,
    /// 请求延迟统计
    #[prost(message, repeated, tag = "2")]
    pub req_reply_statistics_list: ::prost::alloc::vec::Vec<ReqReplyStatisticsItem>,
    /// 下单延迟统计
    #[prost(message, repeated, tag = "3")]
    pub place_order_statistics_list: ::prost::alloc::vec::Vec<PlaceOrderStatisticsItem>,
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
    /// 返回结果，参见Common.RetType的枚举定义
    #[prost(int32, required, tag = "1", default = "-400")]
    pub ret_type: i32,
    /// 返回结果描述
    #[prost(string, optional, tag = "2")]
    pub ret_msg: ::core::option::Option<::prost::alloc::string::String>,
    /// 错误码，客户端一般通过retType和retMsg来判断结果和详情，errCode只做日志记录，仅在个别协议失败时对账用
    #[prost(int32, optional, tag = "3")]
    pub err_code: ::core::option::Option<i32>,
    #[prost(message, optional, tag = "4")]
    pub s2c: ::core::option::Option<S2c>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum DelayStatisticsType {
    /// 未知类型
    Unkonw = 0,
    /// 行情推送统计
    QotPush = 1,
    /// 请求回应统计
    ReqReply = 2,
    /// 下单统计
    PlaceOrder = 3,
}
impl DelayStatisticsType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            DelayStatisticsType::Unkonw => "DelayStatisticsType_Unkonw",
            DelayStatisticsType::QotPush => "DelayStatisticsType_QotPush",
            DelayStatisticsType::ReqReply => "DelayStatisticsType_ReqReply",
            DelayStatisticsType::PlaceOrder => "DelayStatisticsType_PlaceOrder",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "DelayStatisticsType_Unkonw" => Some(Self::Unkonw),
            "DelayStatisticsType_QotPush" => Some(Self::QotPush),
            "DelayStatisticsType_ReqReply" => Some(Self::ReqReply),
            "DelayStatisticsType_PlaceOrder" => Some(Self::PlaceOrder),
            _ => None,
        }
    }
}
/// 某段时间的统计数据
/// SR表示服务器收到数据，目前只有港股支持SR字段，SS表示服务器发出数据
/// CR表示OpenD收到数据，CS表示OpenD发出数据
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum QotPushStage {
    /// 未知
    Unkonw = 0,
    /// 统计服务端处理耗时
    Sr2ss = 1,
    /// 统计网络耗时
    Ss2cr = 2,
    /// 统计OpenD处理耗时
    Cr2cs = 3,
    /// 统计服务器发出到OpenD发出的处理耗时
    Ss2cs = 4,
    /// 统计服务器收到数据到OpenD发出的处理耗时
    Sr2cs = 5,
}
impl QotPushStage {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            QotPushStage::Unkonw => "QotPushStage_Unkonw",
            QotPushStage::Sr2ss => "QotPushStage_SR2SS",
            QotPushStage::Ss2cr => "QotPushStage_SS2CR",
            QotPushStage::Cr2cs => "QotPushStage_CR2CS",
            QotPushStage::Ss2cs => "QotPushStage_SS2CS",
            QotPushStage::Sr2cs => "QotPushStage_SR2CS",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "QotPushStage_Unkonw" => Some(Self::Unkonw),
            "QotPushStage_SR2SS" => Some(Self::Sr2ss),
            "QotPushStage_SS2CR" => Some(Self::Ss2cr),
            "QotPushStage_CR2CS" => Some(Self::Cr2cs),
            "QotPushStage_SS2CS" => Some(Self::Ss2cs),
            "QotPushStage_SR2CS" => Some(Self::Sr2cs),
            _ => None,
        }
    }
}
/// 行情推送类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum QotPushType {
    /// 未知
    Unkonw = 0,
    /// 最新价
    Price = 1,
    /// 逐笔
    Ticker = 2,
    /// 摆盘
    OrderBook = 3,
    /// 经纪队列
    Broker = 4,
}
impl QotPushType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            QotPushType::Unkonw => "QotPushType_Unkonw",
            QotPushType::Price => "QotPushType_Price",
            QotPushType::Ticker => "QotPushType_Ticker",
            QotPushType::OrderBook => "QotPushType_OrderBook",
            QotPushType::Broker => "QotPushType_Broker",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "QotPushType_Unkonw" => Some(Self::Unkonw),
            "QotPushType_Price" => Some(Self::Price),
            "QotPushType_Ticker" => Some(Self::Ticker),
            "QotPushType_OrderBook" => Some(Self::OrderBook),
            "QotPushType_Broker" => Some(Self::Broker),
            _ => None,
        }
    }
}
