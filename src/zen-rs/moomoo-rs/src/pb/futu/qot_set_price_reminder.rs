#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct C2s {
    /// 股票
    #[prost(message, required, tag = "1")]
    pub security: super::qot_common::Security,
    /// SetPriceReminderOp，操作类型
    #[prost(int32, required, tag = "2")]
    pub op: i32,
    /// 到价提醒的标识，GetPriceReminder协议可获得，用于指定要操作的到价提醒项，对于新增的情况不需要填
    #[prost(int64, optional, tag = "3")]
    pub key: ::core::option::Option<i64>,
    /// Qot_Common::PriceReminderType，提醒类型，删除、启用、禁用的情况下会忽略该字段
    #[prost(int32, optional, tag = "4")]
    pub r#type: ::core::option::Option<i32>,
    /// Qot_Common::PriceReminderFreq，提醒频率类型，删除、启用、禁用的情况下会忽略该字段
    #[prost(int32, optional, tag = "7")]
    pub freq: ::core::option::Option<i32>,
    /// 提醒值，删除、启用、禁用的情况下会忽略该字段（精确到小数点后 3 位，超出部分会被舍弃）
    #[prost(double, optional, tag = "5")]
    pub value: ::core::option::Option<f64>,
    /// 用户设置到价提醒时的标注，仅支持 20 个以内的中文字符，删除、启用、禁用的情况下会忽略该字段
    #[prost(string, optional, tag = "6")]
    pub note: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct S2c {
    /// 设置成功的情况下返回对应的key，不成功返回0
    #[prost(int64, required, tag = "1")]
    pub key: i64,
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
pub enum SetPriceReminderOp {
    Unknown = 0,
    /// 新增
    Add = 1,
    /// 删除
    Del = 2,
    /// 启用
    Enable = 3,
    /// 禁用
    Disable = 4,
    /// 修改
    Modify = 5,
    /// 删除该支股票下所有到价提醒
    DelAll = 6,
}
impl SetPriceReminderOp {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            SetPriceReminderOp::Unknown => "SetPriceReminderOp_Unknown",
            SetPriceReminderOp::Add => "SetPriceReminderOp_Add",
            SetPriceReminderOp::Del => "SetPriceReminderOp_Del",
            SetPriceReminderOp::Enable => "SetPriceReminderOp_Enable",
            SetPriceReminderOp::Disable => "SetPriceReminderOp_Disable",
            SetPriceReminderOp::Modify => "SetPriceReminderOp_Modify",
            SetPriceReminderOp::DelAll => "SetPriceReminderOp_DelAll",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "SetPriceReminderOp_Unknown" => Some(Self::Unknown),
            "SetPriceReminderOp_Add" => Some(Self::Add),
            "SetPriceReminderOp_Del" => Some(Self::Del),
            "SetPriceReminderOp_Enable" => Some(Self::Enable),
            "SetPriceReminderOp_Disable" => Some(Self::Disable),
            "SetPriceReminderOp_Modify" => Some(Self::Modify),
            "SetPriceReminderOp_DelAll" => Some(Self::DelAll),
            _ => None,
        }
    }
}
