use diesel::{Connection, SqliteConnection};

pub(crate) mod models;

pub fn establish_connection() -> SqliteConnection {
    let database_url = "./tradingview.db";
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|e| panic!("Error connecting to {} {}", database_url, e))
}
