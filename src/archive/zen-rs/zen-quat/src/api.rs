use actix::{Actor, AsyncContext, ContextFutureSpawner, Handler, StreamHandler, WrapFuture};
use std::cell::RefCell;
use std::cmp::max;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut, Sub};
use std::string::ToString;
use std::sync::Arc;
use std::time::Duration;

use actix_web::web::Json;
use actix_web::{error, get, post, web, Error, HttpRequest, HttpResponse, Responder, Result};
use diesel::prelude::*;
use diesel::ExpressionMethods;
use time::{format_description, OffsetDateTime};
use tokio::time::timeout;
use tracing::{debug, info};

use tws_rs::client::market_data::realtime::{req_mkt_data, ReqMktDataParam};
use tws_rs::contracts::{
    contract_details, sec_def_opt, Contract, ReqSecDefOptParams, SecurityType,
};
use zen_core::objects::enums::{Direction, Freq};

use crate::api::jsonrpc::types::error::code::ErrorCode;
use crate::api::jsonrpc::types::request::RpcRequest;
use crate::api::jsonrpc::types::response::{RpcPayload, RpcResponse};
use crate::api::params::{
    BiInfo, Config, Exchange, HistoryRequest, HistoryResponse, LibrarySymbolInfo, OptionPriceItem,
    OptionPriceRequest, SearchRequest, SearchSymbolResultItem, SymbolRequest, ZenBiDetail,
    ZenRequest, ZenResponse,
};
use crate::broker::ib::IB;
use crate::broker::mixed::MixedBroker;
use crate::db::models::Symbol;
use crate::schema::symbols::dsl::symbols;
use crate::schema::symbols::{screener, symbol};
use actix_web_actors::ws;
use diesel_tracing::sqlite::InstrumentedSqliteConnection;
use serde_json::{json, Value};
use tokio_util::bytes::Buf;

mod jsonrpc;
mod params;

/// Define HTTP actor
struct APIEndpointWs {
    broker: Arc<MixedBroker>,
}

impl Actor for APIEndpointWs {
    type Context = ws::WebsocketContext<Self>;
}

impl APIEndpointWs {
    async fn history_ws(broker: Arc<MixedBroker>, params: HistoryRequest) -> HistoryResponse {
        let symbol_ = params.symbol;
        let contract = Contract::auto_stock(symbol_.as_str());
        let freq = IB::freq_map().get(&params.resolution).unwrap().clone();

        let rs = broker
            .borrow()
            .try_subscribe(
                params.use_local.unwrap_or(false),
                &contract,
                freq,
                params.from,
                params.to,
                params.countback as isize,
                false,
            )
            .await;
        if rs.is_err() {
            return HistoryResponse {
                s: "error".to_string(),
                errmsg: Some(format!("error in get_bars: {}", rs.unwrap_err())),
                t: None,
                c: None,
                h: None,
                l: None,
                v: None,
                o: None,
            };
        }
        let from = OffsetDateTime::from_unix_timestamp(params.from).unwrap();
        let to = OffsetDateTime::from_unix_timestamp(params.to).unwrap();
        let mut index: isize = -1;

        let (mut o, mut c, mut h, mut l) = (vec![], vec![], vec![], vec![]);
        let (mut t, mut v) = (vec![], vec![]);
        let zen = broker
            .borrow()
            .get_czsc(params.use_local.unwrap_or(false), &contract, freq);
        let zen = zen.read().await;
        if params.countback > 0 {
            for (idx, bar) in zen.czsc.bars_raw.iter().enumerate() {
                if bar.borrow().dt <= to {
                    index = idx as isize;
                } else {
                    break;
                }
            }
            for idx in max(index + 1 - (params.countback as isize), 0)..(index + 1) {
                let bar = &zen.czsc.bars_raw[idx as usize];
                o.push(bar.borrow().open);
                c.push(bar.borrow().close);
                h.push(bar.borrow().high);
                l.push(bar.borrow().low);
                t.push(bar.borrow().dt.unix_timestamp());
                v.push(bar.borrow().vol);
            }
        } else {
            for bar in &zen.czsc.bars_raw {
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
        return HistoryResponse {
            s: "ok".to_string(),
            errmsg: None,
            t: Some(t),
            c: Some(c),
            h: Some(h),
            l: Some(l),
            v: Some(v),
            o: Some(o),
        };
    }
}

impl Handler<RpcResponse> for APIEndpointWs {
    type Result = ();

    fn handle(&mut self, msg: RpcResponse, ctx: &mut Self::Context) {
        ctx.text(msg.dump());
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for APIEndpointWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                debug!("msg {:?}", msg);
                ctx.pong(&*msg)
            }
            Ok(ws::Message::Text(text)) => {
                let Ok(RpcRequest {
                    jsonrpc,
                    method,
                    params,
                    id,
                }) = serde_json::from_slice(text.as_ref())
                else {
                    return ctx.text(
                        RpcResponse {
                            payload: ErrorCode::ParseError.into(),
                            ..Default::default()
                        }
                        .dump(),
                    );
                };
                let payload = match method.as_str() {
                    "say_hello" => {
                        let payload = RpcPayload::Result(json!("Hello World"));
                        ctx.text(
                            RpcResponse {
                                jsonrpc,
                                payload,
                                id,
                            }
                            .dump(),
                        );
                    }
                    "history" => {
                        let recipient = ctx.address().recipient();
                        let req = params.unwrap()[0].clone();
                        let broker = self.broker.clone();
                        let future = async move {
                            let rsp = APIEndpointWs::history_ws(
                                broker,
                                serde_json::from_value(req).unwrap(),
                            )
                            .await;
                            recipient
                                .send(RpcResponse {
                                    jsonrpc,
                                    payload: RpcPayload::Result(json!(rsp)),
                                    id,
                                })
                                .await
                                .expect("TODO: panic message");
                        };
                        future.into_actor(self).spawn(ctx);
                    }
                    _ => {}
                };
            }
            Ok(ws::Message::Binary(bin)) => {
                debug!("bin {:?}", bin);
                ctx.binary(bin)
            }
            _ => (),
        }
    }
}

#[get("/ws")]
async fn websocket(
    req: HttpRequest,
    stream: web::Payload,
    z: web::Data<MixedBroker>,
) -> Result<HttpResponse, Error> {
    let resp = ws::start(
        APIEndpointWs {
            broker: z.deref().clone(),
        },
        &req,
        stream,
    );
    debug!("{:?}", resp);
    resp
}

#[get("/datafeed/udf/history")]
pub(super) async fn history(
    req: HttpRequest,
    web::Query(params): web::Query<HistoryRequest>,
    z: web::Data<MixedBroker>,
) -> Result<impl Responder> {
    let symbol_ = params.symbol;
    let contract = Contract::auto_stock(symbol_.as_str());
    let freq = IB::freq_map().get(&params.resolution).unwrap().clone();

    let use_local = req
        .headers()
        .get("Realtime")
        .map(|x| x.to_str().unwrap() == "false")
        .unwrap_or(false);
    let rs = z
        .get_ref()
        .borrow()
        .try_subscribe(
            use_local,
            &contract,
            freq,
            params.from,
            params.to,
            params.countback as isize,
            false,
        )
        .await;
    if rs.is_err() {
        return Ok(Json(HistoryResponse {
            s: "error".to_string(),
            errmsg: Some(format!("error in get_bars: {}", rs.unwrap_err())),
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
    let zen = z.borrow().get_czsc(use_local, &contract, freq);
    let zen = zen.read().await;
    if params.countback > 0 {
        for (idx, bar) in zen.czsc.bars_raw.iter().enumerate() {
            if bar.borrow().dt <= to {
                index = idx as isize;
            } else {
                break;
            }
        }
        for idx in max(index + 1 - (params.countback as isize), 0)..(index + 1) {
            let bar = &zen.czsc.bars_raw[idx as usize];
            o.push(bar.borrow().open);
            c.push(bar.borrow().close);
            h.push(bar.borrow().high);
            l.push(bar.borrow().low);
            t.push(bar.borrow().dt.unix_timestamp());
            v.push(bar.borrow().vol);
        }
    } else {
        for bar in &zen.czsc.bars_raw {
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
    req: HttpRequest,
    web::Query(params): web::Query<SearchRequest>,
    z: web::Data<MixedBroker>,
    conn: web::Data<RefCell<InstrumentedSqliteConnection>>,
) -> Result<impl Responder> {
    use crate::schema::symbols::dsl::*;

    let results = symbols
        .filter(screener.eq(params.exchange.clone()))
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
    let mut obj: Vec<SearchSymbolResultItem> = results
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

    let use_local = req
        .headers()
        .get("Realtime")
        .map(|x| x.to_str().unwrap() == "false")
        .unwrap_or(false);

    if (params.query.contains("TSLA ") || params.query.contains("SPY ")) && !use_local {
        let contract = Contract::stock(params.query.split(" ").next().unwrap());
        let rs = z
            .get_ref()
            .borrow()
            .try_subscribe(
                false,
                &contract,
                Freq::D,
                OffsetDateTime::now_utc()
                    .sub(Duration::from_secs(60 * 60))
                    .unix_timestamp(),
                OffsetDateTime::now_utc().unix_timestamp(),
                100,
                false,
            )
            .await;
        if rs.is_err() {
            return Err(error::ErrorInternalServerError(rs.unwrap_err()));
        }

        let last_price = {
            let zen = z.borrow().get_czsc(false, &contract, Freq::F60);
            let zen = zen.read().await;
            zen.czsc
                .bars_raw
                .last()
                .map(|x| x.borrow().close)
                .unwrap_or(0.0)
        };
        info!("last_price {}", last_price);

        let client = &z.borrow().ib.borrow().client.read().await.clone();
        let client = client.borrow();
        let client_ref = client.as_ref().unwrap();
        let contracts = contract_details(client_ref, &contract).await.unwrap();

        let params = timeout(
            Duration::from_secs(4),
            sec_def_opt(
                client_ref,
                &ReqSecDefOptParams {
                    underlying_symbol: contracts[0].contract.symbol.clone(),
                    fut_fop_exchange: "".to_string(),
                    underlying_sec_type: contracts[0].contract.security_type.to_string(),
                    underlying_con_id: contracts[0].contract.contract_id,
                },
            ),
        )
        .await
        .unwrap_or(Ok(vec![]))
        .unwrap_or(vec![]);
        let params = params
            .iter()
            .filter(|x| x.exchange == "SMART")
            .collect::<Vec<_>>();

        let formatter = format_description::parse("[year][month][day]").unwrap();

        let expirations = params[0].expirations.clone();
        let mut expirations = expirations
            .iter()
            .filter(|x| (**x) >= OffsetDateTime::now_utc().format(&formatter).unwrap())
            .collect::<Vec<_>>();
        expirations.sort();

        for expiration in expirations.iter().take(1) {
            for strike in &params[0].strikes {
                for right in ["P", "C"] {
                    let gap = if params[0].trading_class == "TSLA" {
                        250
                    } else {
                        100
                    };
                    if *strike > last_price as f64 - 10.0
                        && *strike < last_price as f64 + 10.0
                        && ((*strike * 100.0) as i64 % gap == 0)
                    {
                        let option = Contract::option(
                            params[0].trading_class.as_str(),
                            expiration.as_str(),
                            *strike,
                            right,
                            params[0].multiplier.as_str(),
                        );
                        let option = contract_details(client_ref, &option).await;
                        if option.is_ok() {
                            let option = &option.unwrap()[0];
                            obj.push(SearchSymbolResultItem {
                                symbol: option.contract.local_symbol.clone(),
                                full_name: option.contract.local_symbol.clone(),
                                description: option.contract.local_symbol.clone(),
                                exchange: option.contract.exchange.clone(),
                                ticker: format!("option:{}", option.contract.local_symbol),
                                r#type: "option".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(Json(obj))
}

#[get("/datafeed/udf/symbols")]
pub(crate) async fn resolve_symbol(
    web::Query(params): web::Query<SymbolRequest>,
    conn: web::Data<RefCell<InstrumentedSqliteConnection>>,
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
    if syms.is_empty() {
        return Err(error::ErrorInternalServerError(""));
    }
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
    req: HttpRequest,
    Json(params): Json<ZenRequest>,
    z: web::Data<MixedBroker>,
) -> Result<impl Responder> {
    //debug!("zen_element {:?}", params);
    let symbol_ = params.symbol;
    let contract = Contract::auto_stock(symbol_.as_str());
    let freq = IB::freq_map().get(&params.resolution).unwrap().clone();
    let use_local = req
        .headers()
        .get("Realtime")
        .map(|x| x.to_str().unwrap() == "false")
        .unwrap_or(false);
    let rs = z
        .get_ref()
        .borrow()
        .try_subscribe(use_local, &contract, freq, params.from, params.to, 0, false)
        .await;
    let zen = z.borrow().get_czsc(use_local, &contract, freq);
    let zen = zen.read().await;
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
    let mut last_dir = None;
    for bi in &zen.czsc.bi_list {
        if bi.fx_b.dt.unix_timestamp() < params.from {
            continue;
        }
        last_dir = Some(bi.direction.clone());
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
    match last_dir {
        Some(Direction::Up) => {
            let bar = zen
                .czsc
                .bars_ubi
                .iter()
                .skip(1)
                .min_by(|a, b| a.low.partial_cmp(&b.low).unwrap())
                .map(|a| a.clone())
                .unwrap();
            resp.bi.unfinished.push(ZenBiDetail {
                direction: String::from(Direction::Down.as_str()),
                end: bar.low,
                end_ts: bar.dt.unix_timestamp(),
                start: zen.czsc.bars_ubi[1].high,
                start_ts: zen.czsc.bars_ubi[1].dt.unix_timestamp(),
            });
        }
        Some(Direction::Down) => {
            let bar = zen
                .czsc
                .bars_ubi
                .iter()
                .skip(1)
                .max_by(|a, b| a.high.partial_cmp(&b.high).unwrap())
                .map(|a| a.clone())
                .unwrap();
            resp.bi.unfinished.push(ZenBiDetail {
                direction: String::from(Direction::Up.as_str()),
                end: bar.high,
                end_ts: bar.dt.unix_timestamp(),
                start: zen.czsc.bars_ubi[1].low,
                start_ts: zen.czsc.bars_ubi[1].dt.unix_timestamp(),
            });
        }
        _ => {}
    }

    resp.beichi.push(vec![]);
    for bc in &zen.beichi_processor.beichi_tracker {
        if bc.macd_b_dt < params.from {
            continue;
        }
        resp.beichi[0].push(bc.clone());
    }
    Ok(Json(resp))
}

#[post("/ma/option_price")]
async fn option_price(
    web::Json(params): web::Json<OptionPriceRequest>,
    z: web::Data<MixedBroker>,
) -> Result<impl Responder> {
    if params.option.is_empty() {
        return Err(error::ErrorInternalServerError("option empty"));
    }
    let mut result = vec![];
    {
        z.get_ref().borrow().ib.borrow().connect().await;
    }
    let client = &z.borrow().ib.borrow().client.read().await.clone();
    let client = client.borrow();
    let client_ref = client.as_ref().unwrap();

    let option = Contract {
        local_symbol: params.option.clone(),
        security_type: SecurityType::Option,
        exchange: "SMART".to_string(),
        ..Contract::default()
    };
    let details = contract_details(&client_ref, &option).await;
    //info!("details {:?}", details);
    let ticker = req_mkt_data(
        &client_ref,
        &ReqMktDataParam {
            contract: details.unwrap()[0].contract.clone(),
            generic_tick_list: Default::default(),
            snapshot: false,
            regulatory_snapshot: false,
            mkt_data_options: vec![],
        },
    )
    .await;

    let mut delta = 0.0f32;
    let mut opt_price = -1.0f32;
    if ticker.is_ok() {
        let ticker = ticker.unwrap().clone();

        let t = ticker.read().await;
        //info!("opt_compute {:?}", t);

        delta = t
            .as_ref()
            .map(|v| v.opt_compute.as_ref().map(|o| o.delta).unwrap_or(0.0))
            .unwrap_or(0.0) as f32;
        opt_price = t
            .as_ref()
            .map(|v| v.opt_compute.as_ref().map(|o| o.opt_price).unwrap_or(0.0))
            .unwrap_or(0.0) as f32;
    }
    for interval in &params.intervals {
        for ma in &params.ma {
            let freq = IB::freq_map().get(&interval.to_string()).unwrap().clone();
            let contract = Contract::auto_stock(params.symbol.as_str());
            let zen = z.borrow().get_czsc(false, &contract, freq);
            let zen = zen.read().await;
            let smas = zen
                .czsc
                .cache
                .get::<crate::calculate::zen_cache::SMATrackerCache>()
                .unwrap();

            let sma = smas.store.get(ma);

            let ma_val = sma.map(|x| x.ma()).unwrap_or(0.0);
            let last = sma.map(|x| x.last()).unwrap_or(0.0);
            result.push(OptionPriceItem {
                interval: freq,
                price: ma_val,
                delta,
                ma: *ma,
                option_price: (opt_price + (ma_val - last) * delta),
            });
        }
    }

    Ok(Json(result))
}
