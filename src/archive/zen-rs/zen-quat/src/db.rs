use diesel::{Connection, SqliteConnection};
use diesel_tracing::sqlite::InstrumentedSqliteConnection;

pub(crate) mod models;

pub fn establish_connection() -> InstrumentedSqliteConnection {
    let database_url = "./tradingview.db";
    InstrumentedSqliteConnection::establish(&database_url)
        .unwrap_or_else(|e| panic!("Error connecting to {} {}", database_url, e))
}
