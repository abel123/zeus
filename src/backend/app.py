from sanic import Request, Sanic, json
from sanic.response import html
from sanic_ext import Extend

import socketio
from mayim.extension import SanicMayimExtension

from backend.curd.sqllite.model import SymbolExecutor

import logging

from backend.datafeed.api import DataFeed
from backend.datafeed.trading_view import LibrarySymbolInfo, PeriodParams

logging.basicConfig(level=logging.DEBUG)
logging.debug("This will get logged")

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
    exchange = request.args.get("exchange", "NASDAQ")
    type = request.args.get("type", "stock")

    symbols = await DataFeed.search_symbols(
        type=type,
        user_input="%" + user_input + "%",
        exchange=exchange,
        executor=executor,
    )
    logging.debug("symbols ---%s", symbols)
    return json([sym.dict() for sym in symbols])


@app.route("/resolve_symbol")
async def resolve_symbol(request: Request, executor: SymbolExecutor):
    symbol = request.args.get("symbol", "")
    exchange = request.args.get("exchange", "NASDAQ")
    type = request.args.get("type", "stock")

    symbols = await DataFeed.resolve_symbol(
        exchange=exchange,
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
    bars = await DataFeed.get_bars(
        symbol_info=symbol_info, resolution=resolution, period_params=period_params
    )
    return json([bar.model_dump(mode="json", exclude_none=True) for bar in bars])


Extend.register(
    SanicMayimExtension(
        executors=[SymbolExecutor],
        dsn="tradingview.db",
    )
)

if __name__ == "__main__":
    app.run()
