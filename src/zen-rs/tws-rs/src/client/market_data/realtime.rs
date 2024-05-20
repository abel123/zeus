use crate::client::transport::message_bus::Signal;
use crate::contracts::{Contract, TagValue};
use crate::messages::{IncomingMessages};
use crate::{ClientRef, Error};
use cached::proc_macro::cached;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::spawn_local;

mod decoders;
mod encoders;

#[derive(Debug)]
pub struct ReqMktDataParam {
    pub contract: Contract,
    pub generic_tick_list: HashSet<GenericTick>,
    pub snapshot: bool,
    pub regulatory_snapshot: bool,
    pub mkt_data_options: Vec<TagValue>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum GenericTick {
    /// Currently for stocks.
    OptionVolume = 100,
    /// Currently for stocks.
    OptionOpenInterest = 101,
    /// Currently for stocks.
    HistoricalVolatility = 104,
    /// Currently for stocks.
    AverageOptionVolume = 105,
    /// Currently for stocks.
    OptionImpliedVolatility = 106,
    IndexFuturePremium = 162,
    MiscellaneousStats = 165,
    /// Used in TWS P&L computations
    MarkPrice = 221,
    /// Volumes, price, and imbalance
    AuctionValues = 225,
    /// Contains the last trade price, last trade size, last trade time, total volume, VWAP, and
    /// single trade flag.
    RtVolume = 233,
    Shortable = 236,
    Inventory = 256,
    FundamentalRatios = 258,
    RealtimeHistoricalVolatility = 411,
    IbDividends = 456,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(i32)]
#[allow(non_camel_case_types)]
pub enum TickType {
    BID_SIZE = 0,
    BID = 1,
    ASK = 2,
    ASK_SIZE = 3,
    LAST = 4,
    LAST_SIZE = 5,
    HIGH = 6,
    LOW = 7,
    VOLUME = 8,
    CLOSE = 9,
    BID_OPTION = 10,
    ASK_OPTION = 11,
    LAST_OPTION = 12,
    MODEL_OPTION = 13,
    OPEN = 14,
    LOW_13_WEEK = 15,
    HIGH_13_WEEK = 16,
    LOW_26_WEEK = 17,
    HIGH_26_WEEK = 18,
    LOW_52_WEEK = 19,
    HIGH_52_WEEK = 20,
    AVG_VOLUME = 21,
    OPEN_INTEREST = 22,
    OPTION_HISTORICAL_VOL = 23,
    OPTION_IMPLIED_VOL = 24,
    OPTION_BID_EXCH = 25,
    OPTION_ASK_EXCH = 26,
    OPTION_CALL_OPEN_INTEREST = 27,
    OPTION_PUT_OPEN_INTEREST = 28,
    OPTION_CALL_VOLUME = 29,
    OPTION_PUT_VOLUME = 30,
    INDEX_FUTURE_PREMIUM = 31,
    BID_EXCH = 32,
    ASK_EXCH = 33,
    AUCTION_VOLUME = 34,
    AUCTION_PRICE = 35,
    AUCTION_IMBALANCE = 36,
    MARK_PRICE = 37,
    BID_EFP_COMPUTATION = 38,
    ASK_EFP_COMPUTATION = 39,
    LAST_EFP_COMPUTATION = 40,
    OPEN_EFP_COMPUTATION = 41,
    HIGH_EFP_COMPUTATION = 42,
    LOW_EFP_COMPUTATION = 43,
    CLOSE_EFP_COMPUTATION = 44,
    LAST_TIMESTAMP = 45,
    SHORTABLE = 46,
    FUNDAMENTAL_RATIOS = 47,
    RT_VOLUME = 48,
    HALTED = 49,
    BID_YIELD = 50,
    ASK_YIELD = 51,
    LAST_YIELD = 52,
    CUST_OPTION_COMPUTATION = 53,
    TRADE_COUNT = 54,
    TRADE_RATE = 55,
    VOLUME_RATE = 56,
    LAST_RTH_TRADE = 57,
    RT_HISTORICAL_VOL = 58,
    IB_DIVIDENDS = 59,
    BOND_FACTOR_MULTIPLIER = 60,
    REGULATORY_IMBALANCE = 61,
    NEWS_TICK = 62,
    SHORT_TERM_VOLUME_3_MIN = 63,
    SHORT_TERM_VOLUME_5_MIN = 64,
    SHORT_TERM_VOLUME_10_MIN = 65,
    DELAYED_BID = 66,
    DELAYED_ASK = 67,
    DELAYED_LAST = 68,
    DELAYED_BID_SIZE = 69,
    DELAYED_ASK_SIZE = 70,
    DELAYED_LAST_SIZE = 71,
    DELAYED_HIGH = 72,
    DELAYED_LOW = 73,
    DELAYED_VOLUME = 74,
    DELAYED_CLOSE = 75,
    DELAYED_OPEN = 76,
    RT_TRD_VOLUME = 77,
    CREDITMAN_MARK_PRICE = 78,
    CREDITMAN_SLOW_MARK_PRICE = 79,
    DELAYED_BID_OPTION = 80,
    DELAYED_ASK_OPTION = 81,
    DELAYED_LAST_OPTION = 82,
    DELAYED_MODEL_OPTION = 83,
    LAST_EXCH = 84,
    LAST_REG_TIME = 85,
    FUTURES_OPEN_INTEREST = 86,
    AVG_OPT_VOLUME = 87,
    DELAYED_LAST_TIMESTAMP = 88,
    SHORTABLE_SHARES = 89,
    DELAYED_HALTED = 90,
    REUTERS_2_MUTUAL_FUNDS = 91,
    ETF_NAV_CLOSE = 92,
    ETF_NAV_PRIOR_CLOSE = 93,
    ETF_NAV_BID = 94,
    ETF_NAV_ASK = 95,
    ETF_NAV_LAST = 96,
    ETF_FROZEN_NAV_LAST = 97,
    ETF_NAV_HIGH = 98,
    ETF_NAV_LOW = 99,
    SOCIAL_MARKET_ANALYTICS = 100,
    ESTIMATED_IPO_MIDPOINT = 101,
    FINAL_IPO_LAST = 102,
    UNKNOWN = i32::MAX,
}
impl From<i32> for TickType {
    fn from(code: i32) -> TickType {
        match code {
            0 => TickType::BID_SIZE,
            1 => TickType::BID,
            2 => TickType::ASK,
            3 => TickType::ASK_SIZE,
            4 => TickType::LAST,
            5 => TickType::LAST_SIZE,
            6 => TickType::HIGH,
            7 => TickType::LOW,
            8 => TickType::VOLUME,
            9 => TickType::CLOSE,
            10 => TickType::BID_OPTION,
            11 => TickType::ASK_OPTION,
            12 => TickType::LAST_OPTION,
            13 => TickType::MODEL_OPTION,
            14 => TickType::OPEN,
            15 => TickType::LOW_13_WEEK,
            16 => TickType::HIGH_13_WEEK,
            17 => TickType::LOW_26_WEEK,
            18 => TickType::HIGH_26_WEEK,
            19 => TickType::LOW_52_WEEK,
            20 => TickType::HIGH_52_WEEK,
            21 => TickType::AVG_VOLUME,
            22 => TickType::OPEN_INTEREST,
            23 => TickType::OPTION_HISTORICAL_VOL,
            24 => TickType::OPTION_IMPLIED_VOL,
            25 => TickType::OPTION_BID_EXCH,
            26 => TickType::OPTION_ASK_EXCH,
            27 => TickType::OPTION_CALL_OPEN_INTEREST,
            28 => TickType::OPTION_PUT_OPEN_INTEREST,
            29 => TickType::OPTION_CALL_VOLUME,
            30 => TickType::OPTION_PUT_VOLUME,
            31 => TickType::INDEX_FUTURE_PREMIUM,
            32 => TickType::BID_EXCH,
            33 => TickType::ASK_EXCH,
            34 => TickType::AUCTION_VOLUME,
            35 => TickType::AUCTION_PRICE,
            36 => TickType::AUCTION_IMBALANCE,
            37 => TickType::MARK_PRICE,
            38 => TickType::BID_EFP_COMPUTATION,
            39 => TickType::ASK_EFP_COMPUTATION,
            40 => TickType::LAST_EFP_COMPUTATION,
            41 => TickType::OPEN_EFP_COMPUTATION,
            42 => TickType::HIGH_EFP_COMPUTATION,
            43 => TickType::LOW_EFP_COMPUTATION,
            44 => TickType::CLOSE_EFP_COMPUTATION,
            45 => TickType::LAST_TIMESTAMP,
            46 => TickType::SHORTABLE,
            47 => TickType::FUNDAMENTAL_RATIOS,
            48 => TickType::RT_VOLUME,
            49 => TickType::HALTED,
            50 => TickType::BID_YIELD,
            51 => TickType::ASK_YIELD,
            52 => TickType::LAST_YIELD,
            53 => TickType::CUST_OPTION_COMPUTATION,
            54 => TickType::TRADE_COUNT,
            55 => TickType::TRADE_RATE,
            56 => TickType::VOLUME_RATE,
            57 => TickType::LAST_RTH_TRADE,
            58 => TickType::RT_HISTORICAL_VOL,
            59 => TickType::IB_DIVIDENDS,
            60 => TickType::BOND_FACTOR_MULTIPLIER,
            61 => TickType::REGULATORY_IMBALANCE,
            62 => TickType::NEWS_TICK,
            63 => TickType::SHORT_TERM_VOLUME_3_MIN,
            64 => TickType::SHORT_TERM_VOLUME_5_MIN,
            65 => TickType::SHORT_TERM_VOLUME_10_MIN,
            66 => TickType::DELAYED_BID,
            67 => TickType::DELAYED_ASK,
            68 => TickType::DELAYED_LAST,
            69 => TickType::DELAYED_BID_SIZE,
            70 => TickType::DELAYED_ASK_SIZE,
            71 => TickType::DELAYED_LAST_SIZE,
            72 => TickType::DELAYED_HIGH,
            73 => TickType::DELAYED_LOW,
            74 => TickType::DELAYED_VOLUME,
            75 => TickType::DELAYED_CLOSE,
            76 => TickType::DELAYED_OPEN,
            77 => TickType::RT_TRD_VOLUME,
            78 => TickType::CREDITMAN_MARK_PRICE,
            79 => TickType::CREDITMAN_SLOW_MARK_PRICE,
            80 => TickType::DELAYED_BID_OPTION,
            81 => TickType::DELAYED_ASK_OPTION,
            82 => TickType::DELAYED_LAST_OPTION,
            83 => TickType::DELAYED_MODEL_OPTION,
            84 => TickType::LAST_EXCH,
            85 => TickType::LAST_REG_TIME,
            86 => TickType::FUTURES_OPEN_INTEREST,
            87 => TickType::AVG_OPT_VOLUME,
            88 => TickType::DELAYED_LAST_TIMESTAMP,
            89 => TickType::SHORTABLE_SHARES,
            90 => TickType::DELAYED_HALTED,
            91 => TickType::REUTERS_2_MUTUAL_FUNDS,
            92 => TickType::ETF_NAV_CLOSE,
            93 => TickType::ETF_NAV_PRIOR_CLOSE,
            94 => TickType::ETF_NAV_BID,
            95 => TickType::ETF_NAV_ASK,
            96 => TickType::ETF_NAV_LAST,
            97 => TickType::ETF_FROZEN_NAV_LAST,
            98 => TickType::ETF_NAV_HIGH,
            99 => TickType::ETF_NAV_LOW,
            100 => TickType::SOCIAL_MARKET_ANALYTICS,
            101 => TickType::ESTIMATED_IPO_MIDPOINT,
            102 => TickType::FINAL_IPO_LAST,
            _ => TickType::UNKNOWN,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TickOptionComputation {
    pub tick_type: TickType,
    pub implied_vol: f64,
    pub delta: f64,
    pub opt_price: f64,
    pub pv_dividend: f64,
    pub gamma: f64,
    pub vega: f64,
    pub theta: f64,
    pub und_price: f64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TickGeneric {
    pub tick_type: TickType,
    pub value: f64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TickString {
    pub tick_type: TickType,
    pub value: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TickEFP {
    pub tick_type: TickType,
    pub basis_points: f64,
    pub formatted_basis_points: String,
    pub implied_futures_price: f64,
    pub hold_days: i32,
    pub future_last_trade_date: String,
    pub dividend_impact: f64,
    pub dividends_to_last_trade_date: f64,
}
#[derive(Debug, Clone)]
pub struct Ticker {
    pub contract: Contract,
    pub opt_compute: Option<TickOptionComputation>,
}

#[cached(
    result = true,
    time = 7200,
    key = "String",
    convert = r##"{ format!("{:?}", req) }"##
)]
pub async fn req_mkt_data(
    client: &ClientRef,
    req: &ReqMktDataParam,
) -> Result<Arc<RwLock<Option<Ticker>>>, Error> {
    let request_id = client.next_request_id();
    let message = encoders::encode_req_mkt_data(request_id, req);

    let mut resp = client.send(request_id, message).await?;

    let result = Arc::new(RwLock::new(Some(Ticker {
        contract: req.contract.clone(),
        opt_compute: None,
    })));
    let result_clone = result.clone();
    let mut receiver = resp.receiver.take();
    let sender = resp.sender.clone();
    resp.request_id.take();

    spawn_local(async move {
        loop {
            match receiver.as_mut().unwrap().recv().await {
                Some(mut message) => {
                    //debug!("msg {:?}", message);
                    match message.message_type() {
                        IncomingMessages::TickOptionComputation => {
                            let opt_compute =
                                decoders::decode_tick_option_computation_msg(&mut message);
                            if opt_compute.as_ref().unwrap().tick_type == TickType::LAST_OPTION {
                                let mut ticker = result_clone.write().await;
                                ticker
                                    .as_mut()
                                    .map(|x| x.opt_compute = Some(opt_compute.unwrap()));
                            }
                        }
                        _ => {}
                    }
                }
                _ => {
                    break;
                }
            }
        }
        {
            let mut ticker = result_clone.write().await;
            *ticker = None;
        }
        let _ = sender.send(Signal::Request(request_id));
    });

    Ok(result)
}
