from contextlib import asynccontextmanager
import time
from typing import Any, Union
from fastapi import FastAPI, Request, Response, WebSocket
from fastapi.responses import JSONResponse
from jsonrpcserver import Result, Success, async_dispatch, method
from loguru import logger
from broker.ib import adjust
from broker.mixed import Mixed, Resolution
from fastapi.middleware.cors import CORSMiddleware

import curd
from param import ZenElementRequest
import udf


@asynccontextmanager
async def lifespan(app: FastAPI):
    yield
    broker: Mixed = app.state.broker
    broker.ib.cache.clear()
    broker.ib.ib.disconnect()


app = FastAPI(lifespan=lifespan)
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.state.broker = Mixed()


@app.middleware("http")
async def log_requests(request: Request, call_next):
    start_time = time.time()

    response = await call_next(request)

    process_time = (time.time() - start_time) * 1000
    formatted_process_time = "{0:.2f}".format(process_time)
    logger.info(
        f"rid={request.url.path} completed_in={formatted_process_time}ms status_code={response.status_code}"
    )

    return response


@app.post("/zen/elements")
async def zen_elements(req: ZenElementRequest):
    freq = Resolution(req.resolution)

    # logger.debug("req {}, {}", req, freq)
    broker: Mixed = app.state.broker
    listener = await broker.subscribe(req.symbol, freq, req.from_, req.to, None, False)

    return Response(content=listener.zen.json(), media_type=JSONResponse.media_type)


@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    await websocket.accept()
    while True:
        try:
            recv = await websocket.receive_text()
            # logger.debug("recv {}", recv)

            data = await async_dispatch(recv)
            # logger.debug("data {}", data)
            await websocket.send_text(data)
        except Exception as e:
            logger.warning("warn {}", e)
            break


@method
async def history(req: Any) -> Result:
    """
    {'symbol': 'NASDAQ:TSLA', 'resolution': '60', 'from': -903645074,
      'to': -898350727, 'use_local': False, 'countback': 329}
    """

    # logger.debug("req {}", req)
    broker: Mixed = app.state.broker
    freq = Resolution(req["resolution"])

    from_, to = req["from"], req["to"]
    count_back = req["countback"]
    bars = (
        await broker.subscribe(req["symbol"], freq, req["from"], req["to"], None, False)
    ).bars

    if count_back is None:
        ib_bars = [
            bar
            for bar in bars
            if adjust(bar.date).timestamp() >= from_ and adjust(bar.date) <= to
        ]
    else:
        count = 0
        for bar in bars[::-1]:
            if adjust(bar.date).timestamp() > to:
                count += 1
            else:
                break
        # ib_bars = [bar for bar in bars if adjust(bar.date).timestamp() <= to]
        # ib_bars = ib_bars[max(len(ib_bars) - count_back, 0) :]
        ib_bars = bars[max(0, len(bars) - count - count_back) : len(bars) - count]
    t, c, h, l, o, v = [], [], [], [], [], []
    # logger.debug("bars:  ", bars[:3])
    for bar in ib_bars:
        t.append(adjust(bar.date).timestamp())
        c.append(bar.close)
        h.append(bar.high)
        o.append(bar.open)
        v.append(bar.volume)
        l.append(bar.low)
    if len(ib_bars) == 0:
        return Success(
            {
                "s": "no_data",
            }
        )
    return Success({"s": "ok", "t": t, "c": c, "h": h, "l": l, "v": v, "o": o})


@app.get("/datafeed/udf/config")
async def config():
    return udf.Config


@app.get("/datafeed/udf/symbols")
async def resolve_symbol(symbol: str):
    return await udf.resolve_symbol(symbol)


@app.get("/datafeed/udf/search")
async def search_symbol(query: str, type: str, exchange: str):
    symbols = await curd.select_symbols(screener=exchange, type=type, user_input=query)
    # logger.debug("symbols: {}", symbols)

    return [
        {
            "symbol": sym[0].symbol,
            "full_name": sym[0].symbol,
            "description": sym[0].desc,
            "exchange": sym[0].exchange,
            "ticker": sym[0].symbol,
            "type": sym[0].type,
        }
        for sym in symbols
    ]
