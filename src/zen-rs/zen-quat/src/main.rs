#![feature(slice_take)]

use chrono::Local;

use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;

use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{get, rt, web, App, HttpRequest, HttpServer};
use diesel_logger::LoggingConnection;
use futures_util::StreamExt;
use log::LevelFilter;
use tracing_subscriber::fmt::time::ChronoLocal;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{registry, EnvFilter};

use tws_rs::Error;

use crate::db::establish_connection;
use crate::zen_manager::ZenManager;

mod api;
mod calculate;
mod db;
mod schema;
mod zen_manager;

fn main() -> std::io::Result<()> {
    let target = Box::new(File::create("./log/log.txt").expect("Can't create file"));

    env_logger::Builder::new()
        .target(env_logger::Target::Pipe(target))
        .filter(None, LevelFilter::Debug)
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}:{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.args()
            )
        })
        .init();

    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_line_number(true)
            .with_timer(ChronoLocal::rfc_3339())
            .with_env_filter(EnvFilter::from_default_env())
            .finish(),
    )
    .expect("logger init");

    rt::System::new().block_on(
        HttpServer::new(|| {
            let cors = Cors::default()
                .allow_any_origin()
                .allow_any_header()
                .allow_any_method()
                .max_age(3600);
            App::new()
                .wrap(Logger::new("%a  %r %s %b  %T"))
                .wrap(cors)
                .data_factory(|| async { Ok::<_, Error>(Rc::new(RefCell::new(ZenManager::new()))) })
                .data_factory(|| async {
                    let conn = establish_connection();
                    let conn = LoggingConnection::new(conn);

                    Ok::<_, Error>(RefCell::new(conn))
                })
                .service(api::history)
                .service(api::search_symbol)
                .service(api::resolve_symbol)
                .service(api::zen_element)
                .service(api::config)
        })
        .workers(1)
        .bind(("127.0.0.1", 8080))?
        .run(),
    )
}
