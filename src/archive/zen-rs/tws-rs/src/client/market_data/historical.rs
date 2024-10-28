use std::fmt::Debug;

use drop_stream::DropStream;
use time::{Date, OffsetDateTime};
use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::{Stream, StreamExt};
use tracing::{warn};
use zen_core::objects;
use zen_core::objects::enums::Freq;

use crate::client::market_data::historical::decoders::decode_historical_data_update;
use crate::client::transport::message_bus::Signal;
use crate::client::ClientRef;
use crate::contracts::Contract;
use crate::messages::{
    IncomingMessages, OutgoingMessages, RequestMessage, ToField,
};
use crate::{server_versions, Error};

mod decoders;
mod encoders;

/// Bar describes the historical data bar.
#[derive(Clone, Debug)]
pub struct Bar {
    /// The bar's date and time (either as a yyyymmss hh:mm:ss formatted string or as system time according to the request). Time zone is the TWS time zone chosen on login.
    // pub time: OffsetDateTime,
    pub date: OffsetDateTime,
    /// The bar's open price.
    pub open: f64,
    /// The bar's high price.
    pub high: f64,
    /// The bar's low price.
    pub low: f64,
    /// The bar's close price.
    pub close: f64,
    /// The bar's traded volume if available (only available for TRADES)
    pub volume: f64,
    /// The bar's Weighted Average Price (only available for TRADES)
    pub wap: f64,
    /// The number of trades during the bar's timespan (only available for TRADES)
    pub count: i32,
}

impl Bar {
    pub fn to_bar(&self, freq: Freq) -> objects::chan::Bar {
        objects::chan::Bar {
            id: 0,
            dt: self.date,
            freq,
            open: self.open as f32,
            high: self.high as f32,
            low: self.low as f32,
            vol: self.volume as f32,
            amount: 0.0,
            close: self.close as f32,
            cache: Default::default(),
            macd_4_9_9: (0.0, 0.0, 0.0),
        }
    }
}
#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
pub enum BarSize {
    Sec,
    Sec5,
    Sec15,
    Sec30,
    Min,
    Min2,
    Min3,
    Min5,
    Min15,
    Min20,
    Min30,
    Hour,
    Hour2,
    Hour3,
    Hour4,
    Hour8,
    Day,
    Week,
    Month,
}

impl ToString for BarSize {
    fn to_string(&self) -> String {
        match self {
            Self::Sec => "1 sec".into(),
            Self::Sec5 => "5 secs".into(),
            Self::Sec15 => "15 secs".into(),
            Self::Sec30 => "30 secs".into(),
            Self::Min => "1 min".into(),
            Self::Min2 => "2 mins".into(),
            Self::Min3 => "3 mins".into(),
            Self::Min5 => "5 mins".into(),
            Self::Min15 => "15 mins".into(),
            Self::Min20 => "20 mins".into(),
            Self::Min30 => "30 mins".into(),
            Self::Hour => "1 hour".into(),
            Self::Hour2 => "2 hours".into(),
            Self::Hour3 => "3 hours".into(),
            Self::Hour4 => "4 hours".into(),
            Self::Hour8 => "8 hours".into(),
            Self::Day => "1 day".into(),
            Self::Week => "1 week".into(),
            Self::Month => "1 month".into(),
        }
    }
}

impl ToField for BarSize {
    fn to_field(&self) -> String {
        self.to_string()
    }
}

#[derive(Clone, Debug, Copy)]
pub struct TWSDuration {
    value: i32,
    unit: char,
}

impl TWSDuration {
    pub const SECOND: Self = Self::seconds(1);
    pub const DAY: Self = Self::days(1);
    pub const WEEK: Self = Self::weeks(1);
    pub const MONTH: Self = Self::months(1);
    pub const YEAR: Self = Self::years(1);

    pub const fn seconds(seconds: i32) -> Self {
        Self {
            value: seconds,
            unit: 'S',
        }
    }

    pub const fn days(days: i32) -> Self {
        Self {
            value: days,
            unit: 'D',
        }
    }

    pub const fn weeks(weeks: i32) -> Self {
        Self {
            value: weeks,
            unit: 'W',
        }
    }

    pub const fn months(months: i32) -> Self {
        Self {
            value: months,
            unit: 'M',
        }
    }

    pub const fn years(years: i32) -> Self {
        Self {
            value: years,
            unit: 'Y',
        }
    }
}

impl ToString for TWSDuration {
    fn to_string(&self) -> String {
        format!("{} {}", self.value, self.unit)
    }
}

impl ToField for TWSDuration {
    fn to_field(&self) -> String {
        self.to_string()
    }
}
pub trait ToDuration {
    fn seconds(&self) -> TWSDuration;
    fn days(&self) -> TWSDuration;
    fn weeks(&self) -> TWSDuration;
    fn months(&self) -> TWSDuration;
    fn years(&self) -> TWSDuration;
}

impl ToDuration for i32 {
    fn seconds(&self) -> TWSDuration {
        TWSDuration::seconds(*self)
    }

    fn days(&self) -> TWSDuration {
        TWSDuration::days(*self)
    }

    fn weeks(&self) -> TWSDuration {
        TWSDuration::weeks(*self)
    }

    fn months(&self) -> TWSDuration {
        TWSDuration::months(*self)
    }

    fn years(&self) -> TWSDuration {
        TWSDuration::years(*self)
    }
}

#[derive(Debug)]
struct HistogramData {
    pub price: f64,
    pub count: i32,
}

#[derive(Clone, Debug)]
pub struct HistoricalData {
    pub start: OffsetDateTime,
    pub end: OffsetDateTime,
    pub bars: Vec<Bar>,
    pub request_id: Option<i32>,
}

#[derive(Debug)]
pub struct Schedule {
    pub start: OffsetDateTime,
    pub end: OffsetDateTime,
    pub time_zone: String,
    pub sessions: Vec<Session>,
}

#[derive(Debug)]
pub struct Session {
    pub reference: Date,
    pub start: OffsetDateTime,
    pub end: OffsetDateTime,
}

/// The historical tick's description. Used when requesting historical tick data with whatToShow = MIDPOINT
#[derive(Debug)]
pub struct TickMidpoint {
    /// timestamp of the historical tick.
    pub timestamp: OffsetDateTime,
    /// historical tick price.
    pub price: f64,
    /// historical tick size
    pub size: i32,
}

/// The historical tick's description. Used when requesting historical tick data with whatToShow = BID_ASK.
#[derive(Debug)]
pub struct TickBidAsk {
    /// Timestamp of the historical tick.
    pub timestamp: OffsetDateTime,
    /// Tick attributes of historical bid/ask tick.
    pub tick_attribute_bid_ask: TickAttributeBidAsk,
    /// Bid price of the historical tick.
    pub price_bid: f64,
    /// Ask price of the historical tick.
    pub price_ask: f64,
    /// Bid size of the historical tick
    pub size_bid: i32,
    /// ask size of the historical tick
    pub size_ask: i32,
}

#[derive(Debug, PartialEq)]
pub struct TickAttributeBidAsk {
    pub bid_past_low: bool,
    pub ask_past_high: bool,
}

/// The historical last tick's description. Used when requesting historical tick data with whatToShow = TRADES.
#[derive(Debug)]
pub struct TickLast {
    /// Timestamp of the historical tick.
    pub timestamp: OffsetDateTime,
    /// Tick attributes of historical bid/ask tick.
    pub tick_attribute_last: TickAttributeLast,
    /// Last price of the historical tick.
    pub price: f64,
    /// Last size of the historical tick.
    pub size: i32,
    /// Source exchange of the historical tick.
    pub exchange: String,
    /// Conditions of the historical tick. Refer to Trade Conditions page for more details: <https://www.interactivebrokers.com/en/index.php?f=7235>.
    pub special_conditions: String,
}

#[derive(Debug, PartialEq)]
pub struct TickAttributeLast {
    pub past_limit: bool,
    pub unreported: bool,
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum WhatToShow {
    Trades,
    MidPoint,
    Bid,
    Ask,
    BidAsk,
    HistoricalVolatility,
    OptionImpliedVolatility,
    FeeRate,
    Schedule,
}

impl ToString for WhatToShow {
    fn to_string(&self) -> String {
        match self {
            Self::Trades => "TRADES".to_string(),
            Self::MidPoint => "MIDPOINT".to_string(),
            Self::Bid => "BID".to_string(),
            Self::Ask => "ASK".to_string(),
            Self::BidAsk => "BID_ASK".to_string(),
            Self::HistoricalVolatility => "HISTORICAL_VOLATILITY".to_string(),
            Self::OptionImpliedVolatility => "OPTION_IMPLIED_VOLATILITY".to_string(),
            Self::FeeRate => "FEE_RATE".to_string(),
            Self::Schedule => "SCHEDULE".to_string(),
        }
    }
}

impl ToField for WhatToShow {
    fn to_field(&self) -> String {
        self.to_string()
    }
}

impl ToField for Option<WhatToShow> {
    fn to_field(&self) -> String {
        match self {
            Some(what_to_show) => what_to_show.to_string(),
            None => "".into(),
        }
    }
}

// https://interactivebrokers.github.io/tws-api/historical_bars.html#hd_duration
pub async fn historical_data<'a>(
    client: &'a ClientRef,
    contract: &'a Contract,
    end_date: Option<OffsetDateTime>,
    duration: TWSDuration,
    bar_size: BarSize,
    what_to_show: Option<WhatToShow>,
    use_rth: bool,
    keep_up_to_date: bool,
) -> Result<(HistoricalData, impl Stream<Item = Result<Bar, Error>> + 'a), Error> {
    if !contract.trading_class.is_empty() || contract.contract_id > 0 {
        client.check_server_version(
            server_versions::TRADING_CLASS,
            "It does not support contract_id nor trading class parameters when requesting historical data.",
        )?;
    }

    if what_to_show == Some(WhatToShow::Schedule) {
        client.check_server_version(
            server_versions::HISTORICAL_SCHEDULE,
            "It does not support requesting of historical schedule.",
        )?;
    }

    let request_id = client.next_request_id();
    let request = encoders::encode_request_historical_data(
        client.server_version(),
        request_id,
        contract,
        end_date,
        duration,
        bar_size,
        what_to_show,
        use_rth,
        keep_up_to_date,
        Vec::<crate::contracts::TagValue>::default(),
    )?;

    let mut stream = client.send(request_id, request).await?;
    if keep_up_to_date {
        let _ = stream.request_id.take();
    }

    let sender = stream.sender.clone();
    let dropper = move || {
        if keep_up_to_date {
            sender.send(Signal::Request(request_id)).unwrap_or(());
        }
    };

    let mut stream = UnboundedReceiverStream::new(stream.receiver.take().unwrap());
    if let Some(mut message) = stream.next().await {
        let time_zone = if let Some(tz) = client.time_zone {
            tz
        } else {
            warn!("server timezone unknown. assuming UTC, but that may be incorrect!");
            time_tz::timezones::db::UTC
        };
        match message.message_type() {
            IncomingMessages::HistoricalData => {
                let mut msg = decoders::decode_historical_data(
                    client.server_version,
                    time_zone,
                    &mut message,
                )?;
                msg.request_id = Some(request_id);

                if !keep_up_to_date {
                    let (_, rx) = unbounded_channel();
                    stream = UnboundedReceiverStream::new(rx);
                }
                let stream = stream.map(|mut e| {
                    decode_historical_data_update(client.server_version, time_zone, &mut e)
                });
                let stream = DropStream::new(stream, dropper);
                Ok((msg, stream))
            }
            IncomingMessages::Error => Err(Error::Simple(message.peek_string(4))),
            _ => Err(Error::Simple(format!(
                "unexpected message: {:?}",
                message.message_type()
            ))),
        }
    } else {
        Err(Error::Simple(
            "did not receive historical data response".into(),
        ))
    }
}

pub async fn cancel_historical_data(client: &ClientRef, request_id: i32) -> Result<(), Error> {
    if request_id <= 0 {
        return Ok(());
    }
    let mut message = RequestMessage::default();

    message.push_field(&OutgoingMessages::CancelHistoricalData);
    message.push_field(&"1");
    message.push_field(&request_id);

    let x = client.send(request_id, message).await?;

    Ok(())
}
