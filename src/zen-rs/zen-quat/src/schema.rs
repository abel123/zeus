// @generated automatically by Diesel CLI.

diesel::table! {
    symbols (exchange, symbol) {
        screener -> Nullable<Text>,
        #[sql_name = "type"]
        type_ -> Nullable<Text>,
        pricescale -> Nullable<Integer>,
        exchange -> Nullable<Text>,
        symbol -> Nullable<Text>,
        logoid -> Nullable<Text>,
        desc -> Nullable<Text>,
    }
}
