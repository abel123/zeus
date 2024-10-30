import os
import logging
import sys
import zen_core
from app import app
from uvicorn import Config, Server
from loguru import logger
from script import m_screenshot
from utils.logger import InterceptHandler
import asyncio
import click
from script.datafeed import sync_to_local


LOG_LEVEL = logging.getLevelName(os.environ.get("LOG_LEVEL", "DEBUG"))

zen_core.init()


def setup_logging():
    # intercept everything at the root logger
    logging.root.handlers = [InterceptHandler()]
    logging.root.setLevel(LOG_LEVEL)

    # remove every other logger's handlers
    # and propagate to root logger
    for name in logging.root.manager.loggerDict.keys():
        logging.getLogger(name).handlers = []
        logging.getLogger(name).propagate = True

    # configure loguru
    logger.configure(handlers=[{"sink": sys.stdout, "serialize": False}])


@click.group()
def cli():
    pass


@cli.command()
@click.argument("filename")
def sync(filename):
    with open(filename, "r") as f:
        data = f.readlines()
        asyncio.run(sync_to_local(data))


@cli.command()
@click.argument("filename")
def screenshot(filename):
    with open(filename, "r") as f:
        data = f.readlines()
        m_screenshot.excute(data)


@cli.command()
def run():
    server = Server(
        Config("app:app", host="0.0.0.0", log_level=LOG_LEVEL, port=8080),
    )

    # setup logging last, to make sure no library overwrites it
    # (they shouldn't, but it happens)
    setup_logging()
    logging.getLogger("uvicorn.access").disabled = True
    logging.getLogger("uvicorn.protocols.http.httptools_impl").disabled = True
    logging.getLogger("websockets.legacy.protocol").disabled = True

    server.run()


if __name__ == "__main__":
    cli()
