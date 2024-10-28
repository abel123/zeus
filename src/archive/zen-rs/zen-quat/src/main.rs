use chrono::Local;
use std::fs::File;
use std::io::Write;

use clap::{Parser, Subcommand};
use log::LevelFilter;
use notify_rust::{get_bundle_identifier_or_default, set_application};
use tracing::debug;
use tracing_subscriber::fmt::time::ChronoLocal;
use tracing_subscriber::EnvFilter;

use crate::pkg::load_db::load_local_db;
use crate::pkg::screenshot::screenshot;
use crate::serve::serve;

mod api;
mod broker;
mod calculate;
mod db;
pub(crate) mod pkg;
mod schema;
mod serve;
mod utils;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Serve {},
    Screenshot {
        #[arg(
            short,
            long,
            value_name = "WATCHLIST",
            default_value_t = String::from("./script/watchlist.txt")
        )]
        watchlist: String,
        #[arg(
        short,
        long,
        value_name = "OUTDIR",
        default_value_t = String::from("./data")
        )]
        outdir: String,
    },
    UpdateLocal {
        #[arg(
        short,
        long,
        value_name = "WATCHLIST",
        default_value_t = String::from("./script/watchlist.txt")
        )]
        watchlist: String,
        #[arg(
        short,
        long,
        value_name = "DB_FILE",
        default_value_t = String::from("./tradingview.db")
        )]
        db_file: String,

        #[arg(short, long, value_name = "VERIFY", default_value_t = false)]
        check_only: bool,
    },
}

fn main() {
    let target = Box::new(File::create("./log/log.txt").expect("Can't create file"));

    let safari_id = get_bundle_identifier_or_default("iTerm 2");
    set_application(&safari_id).expect("panic");

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

    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Screenshot { watchlist, outdir }) => {
            debug!("watchlist path {:?}", watchlist);
            let res = screenshot(watchlist.clone(), outdir.clone());
            if res.is_err() {
                debug!("err {:?}", res.err())
            }
        }
        Some(Commands::UpdateLocal {
            watchlist,
            db_file,
            check_only,
        }) => {
            let res = load_local_db(watchlist.clone(), db_file.clone(), *check_only);
            if res.is_err() {
                debug!("err {:?}", res.err())
            }
        }
        Some(Commands::Serve {}) => {
            let res = serve();
            if res.is_err() {
                debug!("err {:?}", res.err())
            }
        }
        None => {}
    }
}
