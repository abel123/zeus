import dataclasses
from datetime import datetime
from sanic import Request, Sanic, json
from sanic.response import html
from sanic_ext import Extend

import socketio
from mayim.extension import SanicMayimExtension
from backend.calculate.zen import signal
from backend.calculate.zen.signal.macd import MACDArea
from backend.curd.sqllite.model import SymbolExecutor

import logging

from backend.datafeed.api import DataFeed
from backend.datafeed.trading_view import LibrarySymbolInfo, PeriodParams
from czsc.analyze import CZSC
from czsc.enum import Direction, Freq
from czsc.objects import RawBar

logging.basicConfig(level=logging.DEBUG)
logging.debug("This will get logged")

import asyncio
from rubicon.objc.eventloop import EventLoopPolicy

# Install the event loop policy
asyncio.set_event_loop_policy(EventLoopPolicy())

# Get an event loop, and run it!
loop = asyncio.get_event_loop()

sio = socketio.AsyncServer(async_mode="sanic")
app = Sanic(name="sanic_application")
sio.attach(app)
app.config.CORS_ORIGINS = "*"


async def background_task():
    """Example of how to send server generated events to clients."""
    count = 0
    while True:
        await sio.sleep(10)
        count += 1
        # await sio.emit("my_response", {"data": "Server generated event"})


@app.listener("before_server_start")
def before_server_start(sanic, loop):
    sio.start_background_task(background_task)


@app.after_server_start
async def after_server_start(sanic, loop):
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
    logging.debug("symbols ---%s", symbols)
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
    print(request.json)
    symbol_info = LibrarySymbolInfo(**request.json["symbol_info"])
    resolution = request.json["resolution"]
    period_params = PeriodParams(**request.json["period_params"])
    bars, _ = await DataFeed.get_bars(
        symbol_info=symbol_info, resolution=resolution, period_params=period_params
    )
    return json([bar.model_dump(mode="json", exclude_none=True) for bar in bars])


@app.route("/zen/elements")
async def zen_elements(request: Request):
    symbol = request.args.get("symbol")
    from_ts = request.args.get("from")
    to_ts = int(request.args.get("to"))
    print(datetime.now().timestamp(), "from", from_ts, " to ", to_ts)

    resolution = request.args.get("resolution")
    bars, item = await DataFeed.get_bars(
        symbol_info=LibrarySymbolInfo(
            name=symbol,
            full_name=symbol,
            description="",
            type="",
            session="",
            exchange="",
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
    )

    print(item.macd_signal.bc_records)
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
            "beichi": [
                {
                    "macd_a_dt": int(bc.macd_a_dt.timestamp()),
                    "macd_a_val": bc.macd_a_val,
                    "macd_b_dt": int(bc.macd_b_dt.timestamp()),
                    "macd_b_val": bc.macd_b_val,
                    "direction": "up" if bc.direction == Direction.Up else "down",
                }
                for idx, bc in enumerate(item.macd_signal.bc_records)
            ],
        }
    )


Extend.register(
    SanicMayimExtension(
        executors=[SymbolExecutor],
        dsn="tradingview.db",
    )
)

if __name__ == "__main__":
    app.run()
