use chrono::Local;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use log::LevelFilter;
use tracing::debug;
use tracing::field::debug;
use tracing_subscriber::fmt::time::ChronoLocal;
use tracing_subscriber::EnvFilter;

use zen_quat::pkg::screenshot::screenshot;
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// does testing things
    Screenshot {
        /// Sets a custom config file
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
}

fn main() {
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
        None => {}
    }

    // Continued program logic goes here...
}
