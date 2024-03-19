/// 包的唯一标识，用于回放攻击的识别和保护
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PacketId {
    /// 当前TCP连接的连接ID，一条连接的唯一标识，InitConnect协议会返回
    #[prost(uint64, required, tag = "1")]
    pub conn_id: u64,
    /// 自增序列号
    #[prost(uint32, required, tag = "2")]
    pub serial_no: u32,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProgramStatus {
    /// 当前状态
    #[prost(enumeration = "ProgramStatusType", required, tag = "1")]
    pub r#type: i32,
    /// 额外描述
    #[prost(string, optional, tag = "2")]
    pub str_ext_desc: ::core::option::Option<::prost::alloc::string::String>,
}
/// 返回结果
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum RetType {
    /// 成功
    Succeed = 0,
    /// 失败
    Failed = -1,
    /// 超时
    TimeOut = -100,
    /// 连接断开
    DisConnect = -200,
    /// 未知结果
    Unknown = -400,
    /// 包内容非法
    Invalid = -500,
}
impl RetType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            RetType::Succeed => "RetType_Succeed",
            RetType::Failed => "RetType_Failed",
            RetType::TimeOut => "RetType_TimeOut",
            RetType::DisConnect => "RetType_DisConnect",
            RetType::Unknown => "RetType_Unknown",
            RetType::Invalid => "RetType_Invalid",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "RetType_Succeed" => Some(Self::Succeed),
            "RetType_Failed" => Some(Self::Failed),
            "RetType_TimeOut" => Some(Self::TimeOut),
            "RetType_DisConnect" => Some(Self::DisConnect),
            "RetType_Unknown" => Some(Self::Unknown),
            "RetType_Invalid" => Some(Self::Invalid),
            _ => None,
        }
    }
}
/// 包加密算法
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum PacketEncAlgo {
    /// 富途修改过的AES的ECB加密模式
    FtaesEcb = 0,
    /// 不加密
    None = -1,
    /// 标准的AES的ECB加密模式
    AesEcb = 1,
    /// 标准的AES的CBC加密模式
    AesCbc = 2,
}
impl PacketEncAlgo {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            PacketEncAlgo::FtaesEcb => "PacketEncAlgo_FTAES_ECB",
            PacketEncAlgo::None => "PacketEncAlgo_None",
            PacketEncAlgo::AesEcb => "PacketEncAlgo_AES_ECB",
            PacketEncAlgo::AesCbc => "PacketEncAlgo_AES_CBC",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "PacketEncAlgo_FTAES_ECB" => Some(Self::FtaesEcb),
            "PacketEncAlgo_None" => Some(Self::None),
            "PacketEncAlgo_AES_ECB" => Some(Self::AesEcb),
            "PacketEncAlgo_AES_CBC" => Some(Self::AesCbc),
            _ => None,
        }
    }
}
/// 协议格式，请求协议在请求头中指定，推送协议在Init时指定
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ProtoFmt {
    /// Google Protobuf格式
    Protobuf = 0,
    /// Json格式
    Json = 1,
}
impl ProtoFmt {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ProtoFmt::Protobuf => "ProtoFmt_Protobuf",
            ProtoFmt::Json => "ProtoFmt_Json",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "ProtoFmt_Protobuf" => Some(Self::Protobuf),
            "ProtoFmt_Json" => Some(Self::Json),
            _ => None,
        }
    }
}
/// 用户注册归属地
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum UserAttribution {
    ///
    Unknown = 0,
    /// 大陆
    Nn = 1,
    /// MooMoo
    Mm = 2,
    /// 新加坡
    Sg = 3,
    /// 澳洲
    Au = 4,
    /// 日本
    Jp = 5,
    /// 香港
    Hk = 6,
}
impl UserAttribution {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            UserAttribution::Unknown => "UserAttribution_Unknown",
            UserAttribution::Nn => "UserAttribution_NN",
            UserAttribution::Mm => "UserAttribution_MM",
            UserAttribution::Sg => "UserAttribution_SG",
            UserAttribution::Au => "UserAttribution_AU",
            UserAttribution::Jp => "UserAttribution_JP",
            UserAttribution::Hk => "UserAttribution_HK",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "UserAttribution_Unknown" => Some(Self::Unknown),
            "UserAttribution_NN" => Some(Self::Nn),
            "UserAttribution_MM" => Some(Self::Mm),
            "UserAttribution_SG" => Some(Self::Sg),
            "UserAttribution_AU" => Some(Self::Au),
            "UserAttribution_JP" => Some(Self::Jp),
            "UserAttribution_HK" => Some(Self::Hk),
            _ => None,
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ProgramStatusType {
    None = 0,
    /// 已完成类似加载配置,启动服务器等操作,服务器启动之前的状态无需返回
    Loaded = 1,
    /// 登录中
    Loging = 2,
    /// 需要图形验证码
    NeedPicVerifyCode = 3,
    /// 需要手机验证码
    NeedPhoneVerifyCode = 4,
    /// 登录失败,详细原因在描述返回
    LoginFailed = 5,
    /// 客户端版本过低
    ForceUpdate = 6,
    /// 正在拉取类似免责声明等一些必要信息
    NessaryDataPreparing = 7,
    /// 缺少必要信息
    NessaryDataMissing = 8,
    /// 未同意免责声明
    UnAgreeDisclaimer = 9,
    /// 可以接收业务协议收发,正常可用状态
    Ready = 10,
    /// OpenD登录后被强制退出登录，会导致连接全部断开,需要重连后才能得到以下该状态（并且需要在ui模式下）
    ///
    /// 被强制退出登录,例如修改了登录密码,中途打开设备锁等,详细原因在描述返回
    ForceLogout = 11,
    /// 拉取免责声明标志失败
    DisclaimerPullFailed = 12,
}
impl ProgramStatusType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ProgramStatusType::None => "ProgramStatusType_None",
            ProgramStatusType::Loaded => "ProgramStatusType_Loaded",
            ProgramStatusType::Loging => "ProgramStatusType_Loging",
            ProgramStatusType::NeedPicVerifyCode => "ProgramStatusType_NeedPicVerifyCode",
            ProgramStatusType::NeedPhoneVerifyCode => {
                "ProgramStatusType_NeedPhoneVerifyCode"
            }
            ProgramStatusType::LoginFailed => "ProgramStatusType_LoginFailed",
            ProgramStatusType::ForceUpdate => "ProgramStatusType_ForceUpdate",
            ProgramStatusType::NessaryDataPreparing => {
                "ProgramStatusType_NessaryDataPreparing"
            }
            ProgramStatusType::NessaryDataMissing => {
                "ProgramStatusType_NessaryDataMissing"
            }
            ProgramStatusType::UnAgreeDisclaimer => "ProgramStatusType_UnAgreeDisclaimer",
            ProgramStatusType::Ready => "ProgramStatusType_Ready",
            ProgramStatusType::ForceLogout => "ProgramStatusType_ForceLogout",
            ProgramStatusType::DisclaimerPullFailed => {
                "ProgramStatusType_DisclaimerPullFailed"
            }
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "ProgramStatusType_None" => Some(Self::None),
            "ProgramStatusType_Loaded" => Some(Self::Loaded),
            "ProgramStatusType_Loging" => Some(Self::Loging),
            "ProgramStatusType_NeedPicVerifyCode" => Some(Self::NeedPicVerifyCode),
            "ProgramStatusType_NeedPhoneVerifyCode" => Some(Self::NeedPhoneVerifyCode),
            "ProgramStatusType_LoginFailed" => Some(Self::LoginFailed),
            "ProgramStatusType_ForceUpdate" => Some(Self::ForceUpdate),
            "ProgramStatusType_NessaryDataPreparing" => Some(Self::NessaryDataPreparing),
            "ProgramStatusType_NessaryDataMissing" => Some(Self::NessaryDataMissing),
            "ProgramStatusType_UnAgreeDisclaimer" => Some(Self::UnAgreeDisclaimer),
            "ProgramStatusType_Ready" => Some(Self::Ready),
            "ProgramStatusType_ForceLogout" => Some(Self::ForceLogout),
            "ProgramStatusType_DisclaimerPullFailed" => Some(Self::DisclaimerPullFailed),
            _ => None,
        }
    }
}
