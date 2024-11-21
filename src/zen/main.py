import os
import logging
import sys
import zen_core
from app import app
from uvicorn import Config, Server
from loguru import logger
from script import m_add_toc, m_screenshot, ttm_squeeze_scaner
from utils.logger import InterceptHandler
import asyncio
import click
from script.datafeed import sync_to_local

import nest_asyncio

nest_asyncio.apply()
LOG_LEVEL = logging.getLevelName(os.environ.get("LOG_LEVEL", "INFO"))

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
@click.argument("pdf")
@click.argument("list")
def add_toc(pdf, list):
    m_add_toc.excute(pdf, list)


@cli.command()
@click.argument("filename")
def screenshot(filename):
    with open(filename, "r") as f:
        data = f.readlines()
        m_screenshot.excute(filename.split("/")[-1].split(".")[0], data)


@cli.command()
@click.argument("filename")
@click.option("--nofilter", "-n", type=bool, default=False, is_flag=True)
@click.option("--latest", "-l", type=bool, default=False, is_flag=True)
@click.option("--reverse", "-r", type=bool, default=False, is_flag=True)
@click.option("--sort", "-s", type=str, default="sum")
def ttm(filename, nofilter, latest, reverse, sort):
    with open(filename, "r") as f:
        data = f.readlines()
        asyncio.run(ttm_squeeze_scaner.excute(data, nofilter, latest, reverse, sort))


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
