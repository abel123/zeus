import json
import time
from typing import Any, Union
from fastapi import FastAPI, Request, Response, WebSocket
from fastapi.responses import JSONResponse
from jsonrpcserver import Result, Success, async_dispatch, method
from loguru import logger
import orjson
import zen_core
from broker.mixed import Mixed, Resolution
from fastapi.middleware.cors import CORSMiddleware

from param import ZenElementRequest

app = FastAPI()
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


@app.get("/")
def read_root():
    logger.debug("That's it, beautiful and simple logging!")

    return {"Hello": "World"}


@app.post("/zen/elements")
async def zen_elements(req: ZenElementRequest):
    freq = Resolution(req.resolution)

    logger.debug("req {}, {}", req, freq)
    broker: Mixed = app.state.broker
    listener = await broker.subscribe(req.symbol, freq, req.from_, req.to, None, False)

    return Response(content=listener.zen.json(), media_type=JSONResponse.media_type)


@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    await websocket.accept()
    while True:
        data = await async_dispatch(await websocket.receive())
        await websocket.send(data)


@method
async def history(req: Any) -> Result:
    logger.debug("req {}", req)
    return Success("pong")
