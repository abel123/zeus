from datetime import datetime, timedelta
from loguru import logger
from sanic import Request, Sanic, json
from backend.curd.sqllite.model import SymbolExecutor

from backend.datafeed import tv_model
from backend.datafeed.api import DataFeed


class UDF:
    def __init__(self, app: Sanic) -> None:
        prefix = "/datafeed/udf"
        app.add_route(self.config, prefix + "/config", name="udf_config")
        app.add_route(self.resolve_symbols, prefix + "/symbols", name="udf_symbols")
        app.add_route(self.search_symbol, prefix + "/search", name="udf_search_symbol")
        app.add_route(self.history_bars, prefix + "/history", name="udf_history")
        app.add_route(self.quotes, prefix + "/quotes", name="udf_quotes")

    async def config(self, _: Request):
        return json(
            tv_model.Config(
                supported_resolutions=[
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
                exchanges=[
                    tv_model.Exchange(name="US", value="america", desc="US market"),
                    tv_model.Exchange(
                        name="HK", value="hongkong", desc="Hongkong market"
                    ),
                    tv_model.Exchange(
                        name="CN", value="china", desc="China A stock market"
                    ),
                ],
                supports_search=True,
            ).model_dump(mode="json", exclude_none=True)
        )

    async def resolve_symbols(self, request: Request, executor: SymbolExecutor):
        """
        GET /symbols?symbol=NYSE:MSFT
        """
        symbol: str = request.args.get("symbol", "")
        type = "stock"
        screener="america"
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
        
        logger.debug(f"{screener}== {symbol.split(":")} xxxxxxx {symbol} {type}")
        symbols = await DataFeed.resolve_symbol(
            screener=screener,
            type=type,
            symbol=symbol,
            executor=executor,
        )
        logger.debug(f"symbols --- {symbols}")
        return json(symbols[0].model_dump(mode="json", exclude_none=True))

    async def search_symbol(self, request: Request, executor: SymbolExecutor):
        """
        Request: GET /search?query=<query>&type=<type>&exchange=<exchange>&limit=<limit>

        query: string. Text typed by the user in the Symbol Search edit box
        type: string. One of the symbol types supported by your back-end
        exchange: string. One of the exchanges supported by your back-end
        limit: integer. The maximum number of symbols in a response
        """
        user_input = request.args.get("query", "")
        screener = request.args.get("exchange", "NASDAQ")
        type = request.args.get("type", "stock")

        symbols = await DataFeed.search_symbols(
            type=type,
            user_input="%" + user_input + "%",
            screener=screener,
            executor=executor,
        )
        logger.debug(f"symbols ---{symbols}")
        if symbols is None:
            return json({})
        return json([sym.model_dump() for sym in symbols])

    async def history_bars(self, request: Request):
        """
        GET /history?symbol=<ticker_name>&from=<unix_timestamp>&to=<unix_timestamp>
        &resolution=<resolution>&countback=<countback>
        """
        symbol: str = request.args.get("symbol", "")
        exchange = ""
        if ":" in symbol:
            (
                exchange,
                symbol,
            ) = symbol.split(
                ":"
            )[:2]
        screener = "america"
        symbol_info = tv_model.LibrarySymbolInfo(
            name=symbol,
            ticker=symbol,
            full_name=exchange + ":" + symbol,
            description="",
            exchange=exchange,
            type="stock",
            session=tv_model.Market[screener]["session"],
            timezone=tv_model.Market[screener]["timezone"],
            listed_exchange="",
            format=tv_model.SeriesFormat.price,
            pricescale=1,
            minmov=1,  ###
            minmove2=1,  ###
            supported_resolutions=[
                "1",
                "5",
                "15",
                "30",
                "60",
                "1D",
                "1W",
                "1M",
                "12M",
            ],
            intraday_multipliers=[
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
            seconds_multipliers=[
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
            has_intraday=True,
            has_seconds=False,
            has_daily=True,
            has_weekly_and_monthly=True,
        )
        resolution = request.args.get("resolution")
        period_params = tv_model.PeriodParams(
            **{
                "from": request.args.get("from"),
                "to": request.args.get("to"),
                "countBack": request.args.get("countback"),
                "firstDataRequest": False,
            }
        )
        if period_params.to < 0:
            return json(
                {
                    "s": "no_data",
                }
            )
        
        if period_params.to > datetime.now().timestamp():
            period_params.to = datetime.now().timestamp()
        logger.debug(f"period: {period_params}")

        bars, _ = await DataFeed.get_bars(
            symbol_info=symbol_info, resolution=resolution, period_params=period_params
        )
        if bars is None:
            return json({"s": "error", "errmsg": "error in get_bars"})
        
        t, c, h, l, o, v = [], [], [], [], [], []
        #logger.debug("bars:  ", bars[:3])
        for bar in bars:
            t.append(bar.time)
            c.append(bar.close)
            h.append(bar.high)
            o.append(bar.open)
            v.append(bar.volume)
            l.append(bar.low)
        if len(bars) == 0:
            return json(
                {
                    "s": "no_data",
                }
            )
        return json({"s": "ok", "t": t, "c": c, "h": h, "l": l, "v": v, "o": o})

    def quotes(self, request: Request):
        return json({"s": "error", "errmsg": "unimplemented"})
        ...
