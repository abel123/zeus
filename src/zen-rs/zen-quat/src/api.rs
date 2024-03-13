use std::cell::RefCell;
use std::cmp::max;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::string::ToString;

use actix_web::web::Json;
use actix_web::{error, get, post, web, Error, Responder, Result};
use diesel::internal::derives::multiconnection::SelectStatementAccessor;
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel_logger::LoggingConnection;
use time::OffsetDateTime;
use tokio::sync::oneshot::channel;
use tokio::task::spawn_local;
use tracing::debug;
use tws_rs::client::market_data::historical::cancel_historical_data;

use tws_rs::contracts::Contract;
use zen_core::objects::enums::Freq::F1;
use zen_core::objects::enums::{Direction, Freq};

use crate::api::params::{
    BiInfo, Config, Exchange, HistoryRequest, HistoryResponse, LibrarySymbolInfo, SearchRequest,
    SearchSymbolResultItem, SymbolRequest, ZenBiDetail, ZenRequest, ZenResponse,
};
use crate::db::establish_connection;
use crate::db::models::Symbol;
use crate::schema::symbols::dsl::symbols;
use crate::schema::symbols::{exchange, screener, symbol, type_};
use crate::zen_manager::{AppZenMgr, Store, ZenManager};

mod params;

#[get("/datafeed/udf/history")]
pub(super) async fn history(
    web::Query(params): web::Query<HistoryRequest>,
    z: web::Data<AppZenMgr>,
) -> Result<impl Responder> {
    let mut symbol_ = params.symbol;
    if symbol_.contains(':') {
        symbol_ = symbol_.split(":").collect::<Vec<&str>>()[1].to_string();
    }
    let contract = Contract::stock(symbol_.as_str());
    let freq = ZenManager::freq_map()
        .get(&params.resolution)
        .unwrap()
        .clone();
    let rs =
        ZenManager::try_subscribe(z.get_ref().clone(), &contract, freq, params.from, params.to)
            .await;
    if rs.is_err() {
        return Ok(Json(HistoryResponse {
            s: "error".to_string(),
            errmsg: Some("error in get_bars".to_string()),
            t: None,
            c: None,
            h: None,
            l: None,
            v: None,
            o: None,
        }));
    }
    let from = OffsetDateTime::from_unix_timestamp(params.from).unwrap();
    let to = OffsetDateTime::from_unix_timestamp(params.to).unwrap();
    let mut index: isize = -1;

    let (mut o, mut c, mut h, mut l) = (vec![], vec![], vec![], vec![]);
    let (mut t, mut v) = (vec![], vec![]);
    let zen = z.borrow().get_czsc(&contract, freq);
    if params.countback > 0 {
        for (idx, bar) in zen.borrow().czsc.bars_raw.iter().enumerate() {
            if bar.borrow().dt <= to {
                index = idx as isize;
            } else {
                break;
            }
        }
        for idx in max((index + 1 - (params.countback as isize)), 0)..(index + 1) {
            let bar = &zen.borrow().czsc.bars_raw[idx as usize];
            o.push(bar.borrow().open);
            c.push(bar.borrow().close);
            h.push(bar.borrow().high);
            l.push(bar.borrow().low);
            t.push(bar.borrow().dt.unix_timestamp());
            v.push(bar.borrow().vol);
        }
    } else {
        for bar in &zen.borrow().czsc.bars_raw {
            if bar.borrow().dt < from {
                continue;
            }
            if bar.borrow().dt > to {
                break;
            }
            o.push(bar.borrow().open);
            c.push(bar.borrow().close);
            h.push(bar.borrow().high);
            l.push(bar.borrow().low);
            t.push(bar.borrow().dt.unix_timestamp());
            v.push(bar.borrow().vol);
        }
    }
    Ok(Json(HistoryResponse {
        s: "ok".to_string(),
        errmsg: None,
        t: Some(t),
        c: Some(c),
        h: Some(h),
        l: Some(l),
        v: Some(v),
        o: Some(o),
    }))
}

#[get("/datafeed/udf/search")]
async fn search_symbol(
    web::Query(params): web::Query<SearchRequest>,
    conn: web::Data<RefCell<LoggingConnection<SqliteConnection>>>,
) -> Result<impl Responder> {
    use crate::schema::symbols::dsl::*;

    let results = symbols
        .filter(screener.eq(params.exchange))
        .filter(
            symbol
                .like(format!("{}%", params.query))
                .or(desc.like(format!("%{}%", params.query))),
        )
        .limit(20)
        .select(Symbol::as_select())
        .load(conn.borrow_mut().deref_mut())
        .expect("Error loading posts");
    debug!("db res {:?}", results);
    let obj: Vec<SearchSymbolResultItem> = results
        .iter()
        .map(|e| {
            let e = (*e).clone();
            SearchSymbolResultItem {
                symbol: e.symbol.clone().unwrap(),
                full_name: e.symbol.clone().unwrap(),
                description: e.desc.clone().unwrap(),
                exchange: e.exchange.clone().unwrap(),
                ticker: e.symbol.clone().unwrap(),
                r#type: e.type_.clone().unwrap(),
            }
        })
        .collect();
    Ok(Json(obj))
}

#[get("/datafeed/udf/symbols")]
pub(crate) async fn resolve_symbol(
    web::Query(params): web::Query<SymbolRequest>,
    conn: web::Data<RefCell<LoggingConnection<SqliteConnection>>>,
) -> Result<impl Responder> {
    let mut contract_type = "stock".to_string();
    let mut screener_ = "america".to_string();
    let mut symbol_ = params.symbol;
    let mut exchange_ = "".to_string();
    if symbol_.contains(':') {
        exchange_ = symbol_.split(":").collect::<Vec<&str>>()[0].to_string();
        symbol_ = symbol_.split(":").collect::<Vec<&str>>()[1].to_string();
        if exchange_ == "option" {
            contract_type = "option".to_string();
        } else if exchange_ == "HKEX" {
            screener_ = "hongkong".to_string();
        } else if ["SSE".to_string(), "SZSE".to_string()].contains(&exchange_) {
            screener_ = "china".to_string();
        }
    }

    if symbol_.contains(' ') {
        contract_type = "option".to_string()
    }
    if screener_ != "america" {
        symbol_ = format!("{}:{}", exchange_, symbol_)
    }
    let results: Vec<Symbol> = symbols
        .filter(screener.eq(screener_))
        .filter(symbol.eq(symbol_))
        //.filter(type_.eq(contract_type))
        .limit(20)
        .select(Symbol::as_select())
        .load(conn.borrow_mut().deref_mut())
        .expect("Error loading posts");
    let market = HashMap::from([
        (
            "china".to_string(),
            HashMap::from([
                ("session", "0930-1131,1300-1501"),
                ("timezone", "Asia/Shanghai"),
            ]),
        ),
        (
            "hongkong".to_string(),
            HashMap::from([
                ("session", "0930-1200,1300-1601"),
                ("timezone", "Asia/Shanghai"),
            ]),
        ),
        (
            "america".to_string(),
            HashMap::from([("session", "0900-1631"), ("timezone", "America/New_York")]),
        ),
    ]);
    let syms: Vec<LibrarySymbolInfo> = results
        .iter()
        .map(|sym| {
            let symbol_ = sym.symbol.clone().unwrap();
            let exchange_ = sym.exchange.clone().unwrap();
            let screener_ = sym.screener.clone().unwrap();
            LibrarySymbolInfo {
                name: symbol_.clone(),
                ticker: Some(if screener_ == "america" {
                    format!("{}:{}", exchange_, symbol_).to_string()
                } else {
                    symbol_.clone()
                }),
                full_name: if screener_ == "america" {
                    format!("{}:{}", exchange_, symbol_)
                } else {
                    symbol_.clone()
                },
                description: sym.desc.clone().unwrap(),
                exchange: sym.exchange.clone().unwrap(),
                r#type: sym.type_.clone().unwrap(),
                session: market[&screener_.clone()]["session"].to_string(),
                timezone: market[&screener_]["timezone"].to_string(),
                listed_exchange: exchange_,
                format: "price".to_string(),
                pricescale: sym.pricescale.unwrap_or(1) as f64,
                ..Default::default()
            }
        })
        .collect();
    Ok(Json(syms.into_iter().next().unwrap()))
}

#[get("/datafeed/udf/config")]
pub(crate) async fn config() -> Result<impl Responder> {
    Ok(Json(Config {
        supported_resolutions: Some(
            vec![
                "1", "3", "5", "10", "15", "30", "60", "240", "1D", "1W", "1M",
            ]
            .iter()
            .map(|x| x.to_string())
            .collect(),
        ),
        units: None,
        currency_codes: None,
        supports_marks: false,
        supports_time: false,
        supports_timescale_marks: false,
        exchanges: Some(vec![
            Exchange {
                name: "US".to_string(),
                value: "america".to_string(),
                desc: "US market".to_string(),
            },
            Exchange {
                name: "HK".to_string(),
                value: "hongkong".to_string(),
                desc: "Hongkong market".to_string(),
            },
            Exchange {
                name: "CN".to_string(),
                value: "china".to_string(),
                desc: "China A stock market".to_string(),
            },
        ]),
        supports_search: true,
        symbols_types: None,
        supports_group_request: false,
    }))
}

#[post("/zen/elements")]
pub(crate) async fn zen_element(
    web::Json(params): web::Json<ZenRequest>,
    z: web::Data<AppZenMgr>,
) -> Result<impl Responder> {
    //debug!("zen_element {:?}", params);
    let mut symbol_ = params.symbol;
    if symbol_.contains(':') {
        symbol_ = symbol_.split(":").collect::<Vec<&str>>()[1].to_string();
    }
    let contract = Contract::stock(symbol_.as_str());
    let freq = ZenManager::freq_map()
        .get(&params.resolution)
        .unwrap()
        .clone();
    let rs =
        ZenManager::try_subscribe(z.get_ref().clone(), &contract, freq, params.from, params.to)
            .await;
    let zen = z.borrow().get_czsc(&contract, freq);
    if rs.is_err() {
        return Err(error::ErrorInternalServerError(rs.unwrap_err()));
    }

    let mut resp = ZenResponse {
        bi: BiInfo {
            finished: vec![],
            unfinished: vec![],
        },
        beichi: vec![],
        bar_beichi: vec![],
    };
    for bi in &zen.borrow().czsc.bi_list {
        resp.bi.finished.push(ZenBiDetail {
            direction: String::from(bi.direction.as_str()),
            end: if bi.direction == Direction::Down {
                bi.low()
            } else {
                bi.high()
            },
            end_ts: bi.fx_b.dt.unix_timestamp(),
            start: if bi.direction == Direction::Down {
                bi.high()
            } else {
                bi.low()
            },
            start_ts: bi.fx_a.dt.unix_timestamp(),
        })
    }
    resp.beichi
        .push(zen.borrow().bc_processor.beichi_tracker.clone());
    Ok(Json(resp))
}
