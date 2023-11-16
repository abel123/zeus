import dataclasses
from datetime import datetime
from typing import List
from pydantic import BaseModel, Field
from sanic import Request, Sanic, json
from sanic.response import html
from sanic_ext import Extend

import socketio
from mayim.extension import SanicMayimExtension
from backend.broker.ib.broker import Broker
from backend.broker.ib.options import get_tsla_option_list
from backend.calculate.zen import signal
from backend.calculate.zen.signal.macd import MACDArea
from backend.curd.sqllite.model import SymbolExecutor

import logging

from backend.datafeed.api import DataFeed
from backend.datafeed.tv_model import LibrarySymbolInfo, PeriodParams, RequestParam
from backend.datafeed.udf import UDF
from czsc.enum import Direction, Freq
from backend.utils.logger import InterceptHandler, logger

logging.basicConfig(level=logging.DEBUG)
logging.debug("This will get logged")

import asyncio
from rubicon.objc.eventloop import EventLoopPolicy
from sanic import log

# Install the event loop policy
asyncio.set_event_loop_policy(EventLoopPolicy())

# Get an event loop, and run it!
loop = asyncio.get_event_loop()

sio = socketio.AsyncServer(async_mode="sanic")

d = log.LOGGING_CONFIG_DEFAULTS
d["handlers"]["console"]["class"] = "backend.utils.logger.InterceptHandler"
d["handlers"]["error_console"]["class"] = "backend.utils.logger.InterceptHandler"
d["handlers"]["access_console"]["class"] = "backend.utils.logger.InterceptHandler"
del d["handlers"]["console"]["stream"]
del d["handlers"]["error_console"]["stream"]
del d["handlers"]["access_console"]["stream"]

app = Sanic(name="zeus", log_config=d)
sio.attach(app)
app.config.CORS_ORIGINS = "*"
UDF(app)


@app.listener("before_server_stop")
def before_server_stop(sanic, loop):
    Broker.ib.disconnect()
    ...


@app.after_server_start
async def after_server_start(sanic, loop):
    logger.debug("testing")

    await DataFeed.init()


@app.route("/")
async def index(request):
    with open("app.html") as f:
        return html(f.read())


@sio.event
async def connect(sid, environ):
    await sio.emit("my_response", {"data": "Connected", "count": 0}, room=sid)


@sio.event
def disconnect(sid):
    print("Client disconnected")


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
    logger.debug(request.json)
    param = RequestParam(**request.json)
    symbol, from_ts, to_ts, resolution = (
        param.symbol,
        param.from_,
        param.to,
        param.resolution,
    )

    if to_ts > datetime.now().timestamp():
        to_ts = int(datetime.now().timestamp())
    logger.debug(
        f'{datetime.now().timestamp(), "from", from_ts, " to ", to_ts} param {param}'
    )

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

    # logger.debug(item.macd_signal.bc_records)

    beichi = []
    for idx, config in enumerate(param.macd_config):
        logger.debug(f"config {idx}: {config}")
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
                    for bi in item.czsc.finished_bis
                ],
                "unfinished": [item.czsc.unfinished_bi],
            },
            "beichi": beichi,
            "bar_beichi": [
                list(bc.bar_beichi) for bc in item.macd_signal.bc_records.values()
            ],
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
