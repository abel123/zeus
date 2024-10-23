import curd

Market = {
    "china": {"session": "0930-1131,1300-1501", "timezone": "Asia/Shanghai"},
    "hongkong": {"session": "0930-1200,1300-1601", "timezone": "Asia/Shanghai"},
    "america": {
        "session": "0900-1631",
        "timezone": "America/New_York",
    },
}

Config = {
    "exchanges": [
        {"name": "US", "value": "america", "desc": "US market"},
        {"name": "HK", "value": "hongkong", "desc": "Hongkong market"},
        {"name": "CN", "value": "china", "desc": "China A stock market"},
    ],
    "supported_resolutions": [
        "1",
        "3",
        "5",
        "10",
        "15",
        "30",
        "60",
        "240",
        "1D",
        "1W",
        "1M",
    ],
    "supports_marks": False,
    "supports_time": False,
    "supports_timescale_marks": False,
    "supports_search": True,
    "supports_group_request": False,
}


async def resolve_symbol(symbol: str):
    type = "stock"
    screener = "america"
    exchange = ""
    if ":" in symbol:
        exchange = symbol.split(":")[0]
        symbol = symbol.split(":")[1]
        if exchange == "option":
            type = "option"
        elif exchange == "HKEX":
            screener = "hongkong"
        elif exchange in ["SSE", "SZSE"]:
            screener = "china"

    if " " in symbol:
        type = "option"
    if screener != "america":
        symbol = f"{exchange}:{symbol}"

    sym = await curd.resolve_symbols(screener=screener, type=type, symbol=symbol)
    if len(sym) == 0:
        return {}
    else:
        sym = sym[0]

    return {
        "full_name": (
            f"{sym.exchange}:{sym.symbol}" if screener == "america" else sym.symbol
        ),
        "ticker": (
            f"{sym.exchange}:{sym.symbol}" if screener == "america" else sym.symbol
        ),
        "description": sym.desc,
        "type": type,
        "session": Market[sym.screener]["session"],
        "exchange": sym.exchange,
        "listed_exchange": sym.exchange,
        "timezone": Market[sym.screener]["timezone"],
        "format": "price",
        "pricescale": sym.pricescale,
        "minmov": 1,
        "minmove2": 1,
        "has_intraday": True,
        "supported_resolutions": [
            "1",
            "3",
            "5",
            "10",
            "15",
            "30",
            "60",
            "1D",
            "1W",
            "1M",
            "12M",
        ],
        "intraday_multipliers": [
            "1",
            "2",
            "3",
            "5",
            "10",
            "15",
            "20",
            "30",
            "60",
            "120",
            "240",
        ],
        "has_seconds": False,
        "seconds_multipliers": [
            "1",
            "2",
            "3",
            "5",
            "10",
            "15",
            "20",
            "30",
            "40",
            "50",
            "60",
        ],
        "has_daily": True,
        "has_weekly_and_monthly": True,
        "name": sym.symbol,
    }
