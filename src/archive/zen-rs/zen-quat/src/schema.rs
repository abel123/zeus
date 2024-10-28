// @generated automatically by Diesel CLI.

diesel::table! {
    bar_history (symbol, freq, dt) {
        symbol -> Nullable<Text>,
        freq -> Nullable<Text>,
        dt -> Integer,
        high -> Nullable<Float>,
        low -> Nullable<Float>,
        open -> Nullable<Float>,
        close -> Nullable<Float>,
        volume -> Nullable<Integer>,
    }
}

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

diesel::allow_tables_to_appear_in_same_query!(bar_history, symbols,);
