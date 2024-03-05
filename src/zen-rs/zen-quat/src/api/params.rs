use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub(super) struct HistoryRequest {
    pub symbol: String,
    pub resolution: String,
    pub from: i64,
    pub to: i64,
    pub countback: u32,
    pub firstDataRequest: bool,
}

#[derive(Serialize)]
pub(super) struct HistoryResponse {
    pub s: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errmsg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub h: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub o: Option<Vec<f32>>,
}
#[derive(Deserialize, Debug)]
pub(super) struct SearchRequest {
    pub query: String,
    pub r#type: String,
    pub exchange: String,
}

#[derive(Serialize)]
pub(super) struct SearchSymbolResultItem {
    pub symbol: String,
    pub full_name: String,
    pub description: String,
    pub exchange: String,
    pub ticker: String,
    pub r#type: String,
}

#[derive(Deserialize, Debug)]
pub(super) struct SymbolRequest {
    pub(crate) symbol: String,
}

#[derive(Serialize)]
pub(super) struct LibrarySymbolInfo {
    pub full_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_name: Option<Vec<String>>,
    pub ticker: Option<String>,
    pub description: String,
    pub r#type: String,
    pub session: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_display: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub holidays: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corrections: Option<String>,
    pub exchange: String,
    pub listed_exchange: String,
    pub timezone: String,
    pub format: String,
    pub pricescale: f64,
    pub minmov: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fractional: Option<bool>,
    pub minmove2: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_intraday: Option<bool>,
    pub supported_resolutions: Vec<String>,
    pub intraday_multipliers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_seconds: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_ticks: Option<bool>,
    pub seconds_multipliers: Option<Vec<String>>,
    pub has_daily: Option<bool>,
    pub has_weekly_and_monthly: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_empty_bars: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_no_volume: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_precision: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expired: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_date: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_currency_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_unit_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_conversion_types: Option<Vec<String>>,
    pub name: String,
}

impl Default for LibrarySymbolInfo {
    fn default() -> Self {
        Self {
            full_name: "".to_string(),
            base_name: None,
            ticker: None,
            description: "".to_string(),
            r#type: "".to_string(),
            session: "".to_string(),
            session_display: None,
            holidays: None,
            corrections: None,
            exchange: "".to_string(),
            listed_exchange: "".to_string(),
            timezone: "".to_string(),
            format: "".to_string(),
            pricescale: 0.0,
            minmov: 1,
            fractional: None,
            minmove2: 1,
            has_intraday: Some(true),
            supported_resolutions: (vec![
                "1", "3", "5", "10", "15", "30", "60", "1D", "1W", "1M", "12M",
            ]
            .iter()
            .map(|x| x.to_string())
            .collect()),
            intraday_multipliers: Some(
                vec![
                    "1", "2", "3", "5", "10", "15", "20", "30", "60", "120", "240",
                ]
                .iter()
                .map(|x| x.to_string())
                .collect(),
            ),
            has_seconds: Some(false),
            has_ticks: None,
            seconds_multipliers: Some(
                vec!["1", "2", "3", "5", "10", "15", "20", "30", "40", "50", "60"]
                    .iter()
                    .map(|x| x.to_string())
                    .collect(),
            ),
            has_daily: Some(true),
            has_weekly_and_monthly: Some(true),
            has_empty_bars: None,
            has_no_volume: None,
            volume_precision: None,
            data_status: None,
            expired: None,
            expiration_date: None,
            sector: None,
            industry: None,
            currency_code: None,
            original_currency_code: None,
            unit_id: None,
            original_unit_id: None,
            unit_conversion_types: None,
            name: "".to_string(),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct Exchange {
    pub name: String,
    pub value: String,
    pub desc: String,
}

#[derive(Serialize)]
pub(crate) struct SymbolType {
    pub name: String,
    pub value: String,
}

#[derive(Serialize)]
pub(crate) struct Unit {
    pub name: String,
    pub value: String,
    pub desc: String,
}

#[derive(Serialize)]
pub(crate) struct Config {
    pub exchanges: Option<Vec<Exchange>>,
    pub supported_resolutions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub units: Option<HashMap<String, Vec<Unit>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_codes: Option<Vec<String>>,
    pub supports_marks: bool,
    pub supports_time: bool,
    pub supports_timescale_marks: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols_types: Option<Vec<SymbolType>>,
    pub supports_search: bool,
    pub supports_group_request: bool,
}

#[derive(Deserialize, Debug)]
pub(super) struct MacdConfig {
    fast: u32,
    signal: u32,
    slow: u32,
}
#[derive(Deserialize, Debug)]
pub(super) struct ZenRequest {
    pub(crate) symbol: String,
    pub(crate) from: i64,
    pub(crate) to: i64,
    pub(crate) resolution: String,
    pub(crate) macd_config: Vec<MacdConfig>,
}

#[derive(Serialize, Debug)]
pub(super) struct ZenBiDetail {
    pub direction: String,
    pub end: f32,
    pub end_ts: i64,
    pub start: f32,
    pub start_ts: i64,
}
#[derive(Serialize, Debug)]
pub(super) struct BiInfo {
    pub finished: Vec<ZenBiDetail>,
    pub unfinished: Vec<ZenBiDetail>,
}
#[derive(Serialize, Debug)]
pub(super) struct ZenResponse {
    pub bi: BiInfo,
    pub beichi: Vec<()>,
    pub bar_beichi: Vec<()>,
}
