# zeus

chanlun based quant trading

### reference

- 核心逻辑使用的是 https://github.com/waditu/czsc ，原项目主要是为了离线处理。为了方便实时展示，作了少量改动

- script to get assets list

```
curl --location 'https://scanner.tradingview.com/global/scan' \
--header 'Content-Type: application/json' \
--data '{
    "filter": [
        {
            "left": "active_symbol",
            "operation": "equal",
            "right": true
        }
    ],
    "options": {
        "lang": "zh"
    },
    "markets": [
        "china"
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
        45000
    ]
}
'
```
