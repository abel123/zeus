import asyncio
import dataclasses
from datetime import datetime, timedelta
import time
from typing import List
from sanic import Request, Sanic, json
from sanic.response import html
from sanic_ext import Extend

import socketio
from mayim.extension import SanicMayimExtension
from backend.broker.futu.broker import Broker as FutuBroker

from backend.broker.ib.subscribe_manager import SubscribeManager
from backend.calculate.custom.fx_check import FxCheck
from backend.calculate.custom.ma_hit import MAHit
from backend.calculate.custom.macd_area import MACDArea
from backend.calculate.custom.matcher import DefaultMatcher
from backend.calculate.custom.mix_in import ContractSignals, MultipleContractSignals
from backend.calculate.custom.quanty_price_reverse import QPReverseSignals
from backend.calculate.protocol import Symbol, SymbolType
from backend.curd.sqllite.model import SymbolExecutor

import logging

from backend.datafeed.api import DataFeed
from backend.datafeed.tv_model import LibrarySymbolInfo, PeriodParams, RequestParam
from backend.datafeed.udf import UDF
from czsc.enum import Direction, Freq
from backend.utils.logger import logger

from sanic import log
import nest_asyncio

nest_asyncio.apply()

d = log.LOGGING_CONFIG_DEFAULTS
d["handlers"]["console"]["class"] = "backend.utils.logger.InterceptHandler"
d["handlers"]["error_console"]["class"] = "backend.utils.logger.InterceptHandler"
d["handlers"]["access_console"]["class"] = "backend.utils.logger.InterceptHandler"
del d["handlers"]["console"]["stream"]
del d["handlers"]["error_console"]["stream"]
del d["handlers"]["access_console"]["stream"]

app = Sanic(name="zeus", log_config=d)
app.config.CORS_ORIGINS = "*"
UDF(app)


@app.listener("before_server_stop")
def before_server_stop(sanic, loop):
    SubscribeManager().ib.disconnect()
    asyncio.ensure_future(FutuBroker.close())
    time.sleep(1)


@app.after_server_start
async def after_server_start(sanic, loop):
    asyncio.ensure_future(DataFeed.init())

    cs_f1 = ContractSignals(
        Symbol(raw="TSLA", type=SymbolType.STOCK),
        Freq.F1,
        [QPReverseSignals(), MACDArea(), FxCheck()],
    )
    cs_f3 = ContractSignals(
        Symbol(raw="TSLA", type=SymbolType.STOCK),
        Freq.F3,
        [MACDArea(), FxCheck()],
    )
    cs_f5 = ContractSignals(
        Symbol(raw="TSLA", type=SymbolType.STOCK),
        Freq.F5,
        [MACDArea(), FxCheck()],
    )
    cs_f10 = ContractSignals(
        Symbol(raw="TSLA", type=SymbolType.STOCK),
        Freq.F10,
        [MACDArea(), FxCheck()],
    )
    cs_f15 = ContractSignals(
        Symbol(raw="TSLA", type=SymbolType.STOCK),
        Freq.F15,
        [MACDArea(), FxCheck()],
    )
    cs_f30 = ContractSignals(
        Symbol(raw="TSLA", type=SymbolType.STOCK),
        Freq.F30,
        [MACDArea(), MAHit(), FxCheck()],
    )
    cs_f60 = ContractSignals(
        Symbol(raw="TSLA", type=SymbolType.STOCK),
        Freq.F60,
        [MAHit(), FxCheck()],
    )
    # cs_f5, cs_f15, cs_f30, cs_f10, cs_f60
    mcs = MultipleContractSignals(
        [cs_f1, cs_f3, cs_f5, cs_f15, cs_f30, cs_f10, cs_f60],
        DefaultMatcher.match,
    )
    SubscribeManager().upsert_watcher(mcs)


@app.middleware("request")
async def add_start_time(request):
    """Add start time."""
    request.ctx.start_time = time.perf_counter()


@app.middleware("response")
async def write_access_log(request, response):
    if request.method == "OPTIONS":
        return
    spent_time = round((time.perf_counter() - request.ctx.start_time) * 1000)
    remote = f"{request.remote_addr or request.ip}:{request.port}"
    if response.body:
        body_byte = len(response.body)
    else:
        body_byte = 0
    logger.info(
        f"{remote} {request.method} {request.url} {response.status} {body_byte} {spent_time}ms",
    )


app.static("/static", "./static")


@app.route("/search_symbol")
async def search_symbol(request: Request, executor: SymbolExecutor):
    user_input = request.args.get("user_input", "")
    screener = request.args.get("exchange", "NASDAQ")
    type = request.args.get("type", "stock")

    symbols = await DataFeed.search_symbols(
        type=type,
        user_input="%" + user_input + "%",
        screener=screener,
        executor=executor,
    )
    logger.debug("symbols ---%s", symbols)
    return json([sym.dict() for sym in symbols])


@app.route("/resolve_symbol")
async def resolve_symbol(request: Request, executor: SymbolExecutor):
    symbol: str = request.args.get("symbol", "")
    if ":" in symbol:
        symbol = symbol.split(":")[1]
    screener = request.args.get("exchange", "america")
    type = request.args.get("type", "stock")

    symbols = await DataFeed.resolve_symbol(
        screener=screener,
        type=type,
        symbol=symbol,
        executor=executor,
    )
    logging.debug("symbols ---%s", symbols)
    return json(symbols[0].model_dump(mode="json", exclude_none=True))


@app.route("/get_bars", methods=["POST"])
async def get_bars(request: Request):
    logger.debug(request.json)
    symbol_info = LibrarySymbolInfo(**request.json["symbol_info"])
    resolution = request.json["resolution"]
    period_params = PeriodParams(**request.json["period_params"])
    bars, _ = await DataFeed.get_bars(
        symbol_info=symbol_info, resolution=resolution, period_params=period_params
    )
    return json([bar.model_dump(mode="json", exclude_none=True) for bar in bars])


@app.route("/zen/elements", methods=["POST"])
async def zen_elements(request: Request):
    param = RequestParam(**request.json)
    symbol, from_ts, to_ts, resolution = (
        param.symbol,
        param.from_,
        param.to,
        param.resolution,
    )

    if to_ts > datetime.now().timestamp():
        to_ts = int(datetime.now().timestamp())

    bars, item = await DataFeed.get_bars(
        symbol_info=LibrarySymbolInfo(
            name=symbol.split(":")[1] if ":" in symbol else symbol,
            full_name=symbol,
            description="",
            type="",
            session="",
            exchange=symbol.split(":")[0] if ":" in symbol else "",
            listed_exchange="",
            timezone="",
            format="price",
            pricescale=100,
            minmov=1,
            minmove2=100,
            supported_resolutions=[],
        ),
        resolution=resolution,
        period_params=PeriodParams(
            **{
                "from": from_ts,
                "to": to_ts,
                "firstDataRequest": False,
            }
        ),
        macd_config=param.macd_config,
    )
    if bars is None or item is None:
        return json(f"bars is None: {bars is None}, item is None: {item is None}")
    # logger.debug(item.macd_signal.bc_records)

    beichi = []
    for config in param.macd_config:
        if config in item.macd_signal.bc_records:
            beichi.append(
                [
                    {
                        "macd_a_dt": int(bc.macd_a_dt.timestamp()),
                        "macd_a_val": bc.macd_a_val,
                        "macd_b_dt": int(bc.macd_b_dt.timestamp()),
                        "macd_b_val": bc.macd_b_val,
                        "direction": "up" if bc.direction == Direction.Up else "down",
                        "start": {
                            "left_dt": bc.bi_a.sdt.timestamp(),
                            "right_dt": bc.bi_a.edt.timestamp(),
                        },
                        "end": {
                            "left_dt": bc.bi_b.sdt.timestamp(),
                            "right_dt": bc.bi_b.edt.timestamp(),
                        },
                        "high": bc.high,
                        "low": bc.low,
                        "type": bc.type.value,
                    }
                    for bc in item.macd_signal.bc_records[config].values()
                ]
            )
        else:
            logger.debug(f"{config} not find")
            beichi.append([])

    return json(
        {
            "bi": {
                "finished": [
                    {
                        "start_ts": int(bi.sdt.timestamp()),
                        "end_ts": int(bi.edt.timestamp()),
                        "start": bi.high if bi.direction == Direction.Down else bi.low,
                        "end": bi.low if bi.direction == Direction.Down else bi.high,
                        "direction": str(bi.direction),
                    }
                    for bi in item.czsc.bi_list
                ],
                "unfinished": [item.czsc.unfinished_bi]
                if item.czsc.unfinished_bi is not None
                else [],
            },
            "beichi": beichi,
            "bar_beichi": [],
        },
        default=str,
    )


Extend.register(
    SanicMayimExtension(
        executors=[SymbolExecutor],
        dsn="tradingview.db",
    )
)

if __name__ == "__main__":
    app.run()
