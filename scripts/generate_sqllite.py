import sqlite3
import requests

# Crete db if it doesn't exist
with sqlite3.connect("tradingview.db") as con:
    db = con.cursor()
    db.execute(
        "CREATE TABLE IF NOT EXISTS symbols (screener TEXT, type TEXT, pricescale INT, exchange TEXT, symbol TEXT, logoid TEXT, desc TEXT)"
    )
    con.commit()

# Delete all rows
with sqlite3.connect("tradingview.db") as con:
    db = con.cursor()
    db.execute("DELETE FROM symbols")
    con.commit()

SCREENER = {
    "america": ["United States", "en"],
    # "hongkong": ["Hong Kong", "zh"],
    # "china": ["China", "zh"],
}

for x, config in SCREENER.items():
    print(f"Loading screener: {x}")
    r = requests.post(
        f"https://scanner.tradingview.com/{x}/scan",
        data="""
{
    "filter": [
        {
            "left": "active_symbol",
            "operation": "equal",
            "right": true
        }
    ],
    "options": {
        "lang": "%s"
    },
    "markets": [
        "%s"
    ],
    "symbols": {
        "query": {
            "types": []
        },
        "tickers": []
    },
    "columns": [
        "logoid",
        "name",
        "close",
        "change",
        "change_abs",
        "Recommend.All",
        "volume",
        "Value.Traded",
        "market_cap_basic",
        "price_earnings_ttm",
        "earnings_per_share_basic_ttm",
        "number_of_employees",
        "sector",
        "description",
        "type",
        "subtype",
        "update_mode",
        "pricescale",
        "minmov",
        "fractional",
        "minmove2",
        "currency",
        "fundamental_currency_code"
    ],
    "sort": {
        "sortBy": "market_cap_basic",
        "sortOrder": "desc"
    },
    "price_conversion":{"to_symbol":false},
    "range": [
        0,
        450000
    ]
}
"""
        % (config[1], config[0]),
    )
    data = []
    for res in r.json()["data"]:
        exchange, symbol = res["s"].split(":")
        name, type, desc, pricescale = (
            res["d"][0],
            res["d"][14],
            res["d"][13],
            res["d"][17],
        )

        if x != "america":
            symbol = res["s"]
        data.append(
            (
                x,
                type,
                pricescale,
                exchange,
                symbol,
                name,
                desc,
            )
        )

    # Use bulk operation for faster insert
    with sqlite3.connect("tradingview.db") as con:
        con.executemany("INSERT INTO symbols VALUES (?, ?, ?, ?, ?, ?, ?)", data)
