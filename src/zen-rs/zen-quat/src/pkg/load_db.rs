use crate::db::models::BarHistory;
use crate::pkg::screenshot::parse_contract;
use anyhow::Result;
use diesel::{
    Connection, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection,
};
use diesel_tracing::sqlite::InstrumentedSqliteConnection;
use futures_util::future::join_all;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;
use std::time::Duration;
use time::OffsetDateTime;
use tokio::sync::Semaphore;
use tokio::task::{spawn_local, LocalSet};
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info};
use tws_rs::client::market_data::historical::{historical_data, BarSize, TWSDuration, WhatToShow};
use tws_rs::contracts::contract_details_no_cache;
use tws_rs::{Client, Error};
use zen_core::objects::enums::Freq;

pub fn load_local_db(watchlist: String, db_file: String, verify_only: bool) -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let localset = LocalSet::new();
    let res = localset.block_on(&rt, async {
        //LoggingConnection::new(SqliteConnection::establish(&db_file).expect("db exist"));

        let sem = Semaphore::new(1);

        let mut client = Client::new("127.0.0.1:14001", 33332 + rand::random::<i32>() % 100);
        info!("connecting to TWS");
        let client_ref = client.connect().await?;

        spawn_local(async move {
            let callback = move |m| {};
            client.blocking_process(callback).await?;
            Ok::<(), Error>(())
        });
        let client = Rc::new(client_ref);

        let ct = fs::read_to_string(watchlist)?;
        let mut handles = vec![];

        for (idx, line) in ct.lines().enumerate() {
            let contract = parse_contract(line);
            if contract.is_none() {
                continue;
            }
            let contract = contract.unwrap();
            debug!("line {}-{}", idx, line);
            debug!("crawling {}-{}", contract.exchange, contract.symbol);
            if verify_only {
                info!("checking {}-{}", contract.exchange, contract.symbol);
                let res = timeout(
                    Duration::from_secs(3),
                    contract_details_no_cache(&client, &contract),
                )
                .await;
                if res.is_err() {
                    info!("unverified: {} {:?}", line, res);
                } else {
                    debug!("{}: {:?}", line, res);
                }
                sleep(Duration::from_millis(100)).await;
                continue;
            }

            let db_file = db_file.clone();
            let client = client.clone();
            handles.push(spawn_local(async move {
                for (bar_size, (duration, mfreq)) in HashMap::from([
                    (BarSize::Min3, (TWSDuration::days(8), Freq::F3)),
                    (BarSize::Min5, (TWSDuration::months(1), Freq::F5)),
                    (BarSize::Min15, (TWSDuration::months(2), Freq::F15)),
                    (BarSize::Hour, (TWSDuration::months(7), Freq::F60)),
                    (BarSize::Day, (TWSDuration::years(4), Freq::D)),
                    (BarSize::Week, (TWSDuration::years(10), Freq::W)),
                ]) {
                    for _ in 0..2 {
                        let bars = {
                            use crate::schema::bar_history::dsl::*;
                            let mut db = InstrumentedSqliteConnection::establish(&db_file.clone())
                                .expect("db exist");

                            bar_history
                                .filter(symbol.eq(contract.symbol.clone()))
                                .filter(freq.eq(mfreq.as_str()))
                                .order(dt.desc())
                                .limit(3)
                                .select(BarHistory::as_select())
                                .load(&mut db)
                                .expect("TODO: panic message")
                        };
                        let last_bar = if bars.len() < 2 {
                            bars.last()
                        } else {
                            bars.get(bars.len() - 2)
                        };
                        debug!("last bar {:?}", last_bar);
                        let duration = if last_bar.is_some() {
                            timedelta_to_duration(
                                time::Duration::seconds(
                                    OffsetDateTime::now_utc().unix_timestamp()
                                        - (bars.last().unwrap().dt as i64),
                                )
                                .max(time::Duration::days(1)),
                            )
                        } else {
                            duration
                        };
                        let res = timeout(
                            Duration::from_secs(60),
                            historical_data(
                                client.as_ref(),
                                &contract,
                                Option::from(OffsetDateTime::now_utc()),
                                duration,
                                bar_size,
                                Some(WhatToShow::Trades),
                                true,
                                false,
                            ),
                        )
                        .await
                        .map_err(|e| Error::Simple(e.to_string()));
                        if let Ok(Ok((bars, _))) = res {
                            let mut db =
                                SqliteConnection::establish(&db_file.clone()).expect("db exist");

                            if last_bar.is_some() {
                                let mut ok = false;
                                let last_bar = last_bar.unwrap();
                                for bar in &bars.bars {
                                    if (bar.date.unix_timestamp() as i32) < last_bar.dt {
                                        continue;
                                    } else if (bar.date.unix_timestamp() as i32) == last_bar.dt {
                                        if Some(bar.open as f32) == last_bar.open
                                            && Some(bar.close as f32) == last_bar.close
                                        {
                                            ok = true;
                                        }
                                    } else {
                                        break;
                                    }
                                }
                                if !ok {
                                    use crate::schema::bar_history::dsl::*;
                                    let _ = diesel::delete(
                                        bar_history
                                            .filter(symbol.eq(contract.symbol.clone()))
                                            .filter(freq.eq(mfreq.as_str())),
                                    )
                                    .execute(&mut db);
                                    continue;
                                }
                            }
                            use crate::schema::bar_history::dsl::*;
                            let vals = bars
                                .bars
                                .iter()
                                .map(|x| {
                                    (
                                        symbol.eq(contract.symbol.clone()),
                                        freq.eq(mfreq.as_str().to_string()),
                                        dt.eq(x.date.unix_timestamp() as i32),
                                        high.eq(x.high as f32),
                                        low.eq(x.low as f32),
                                        open.eq(x.open as f32),
                                        close.eq(x.close as f32),
                                        volume.eq(x.volume as i32),
                                    )
                                })
                                .collect::<Vec<_>>();
                            diesel::insert_or_ignore_into(bar_history)
                                .values(vals)
                                .execute(&mut db)
                                .map_err(|e| Error::Simple(e.to_string()))
                                .expect("sss");
                            break;
                        }
                    }
                }
            }));
            if handles.len() > 5 {
                let hs = handles.drain(0..);
                join_all(hs).await;
            }
        }

        if handles.len() > 0 {
            let hs = handles.drain(0..);
            join_all(hs).await;
        }
        Ok::<(), Error>(())
    });
    if res.is_err() {
        error!("error: {}", res.unwrap_err())
    }
    Ok(())
}

pub fn timedelta_to_duration(duration: time::Duration) -> TWSDuration {
    if duration.as_seconds_f32() >= time::Duration::days(360).as_seconds_f32() {
        return TWSDuration::years(
            (duration.as_seconds_f32() / time::Duration::days(365).as_seconds_f32()).ceil() as i32,
        );
    } else if duration.as_seconds_f32() >= time::Duration::days(36).as_seconds_f32() {
        return TWSDuration::months(
            (duration.as_seconds_f32() / time::Duration::days(30).as_seconds_f32()).ceil() as i32,
        );
    } else if duration.as_seconds_f32() >= time::Duration::days(7).as_seconds_f32() {
        return TWSDuration::weeks(
            (duration.as_seconds_f32() / time::Duration::days(7).as_seconds_f32()).ceil() as i32,
        );
    } else if duration.as_seconds_f32() >= time::Duration::days(1).as_seconds_f32() {
        return TWSDuration::days(
            (duration.as_seconds_f32() / time::Duration::days(1).as_seconds_f32()).ceil() as i32
                + 1,
        );
    } else {
        return TWSDuration::seconds(duration.as_seconds_f32().ceil() as i32);
    }
}
