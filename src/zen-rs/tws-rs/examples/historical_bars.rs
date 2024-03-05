use std::time::Duration;

use tokio::task::LocalSet;
use tokio::time::{Instant, sleep};
use tokio_stream::StreamExt;
use tracing::info;
use tracing_subscriber;
use tracing_subscriber::EnvFilter;

use tws_rs::client::Client;
use tws_rs::client::market_data::historical::{BarSize, historical_data, TWSDuration, WhatToShow};
use tws_rs::contracts::Contract;
use tws_rs::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_line_number(true)
            .with_env_filter(EnvFilter::from_default_env())
            .finish(),
    );
    
    let mut client = Client::new("127.0.0.1:14001", 4322);
    let client_ref = client.connect().await?;

    tokio::spawn(async move {
        sleep(Duration::from_secs(5)).await;
        let now = Instant::now();
        let (bars, mut stream) = historical_data(
            &client_ref,
            &Contract::stock("TSLA"),
            None,
            TWSDuration::seconds(60),
            BarSize::Min3,
            Some(WhatToShow::Trades),
            true,
            true,
        )
        .await
        .unwrap();
        info!("cost {:?}, bars: {:?}", now.elapsed(), bars);//&bars.bars[..3]);

        while let Some(e) = stream.next().await {
            info!("msg {:?}", e);
        }
    });
    let local = LocalSet::new();
    let res = local
        .run_until(async move {
            client.blocking_process().await?;
            sleep(Duration::from_secs(5)).await;
            Result::<(), Error>::Ok(())
        })
        .await;
    info!("{:?}", res);
    Ok(())
}
