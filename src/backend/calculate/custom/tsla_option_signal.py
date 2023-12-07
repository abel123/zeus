import asyncio

from loguru import logger
from backend.broker.ib.options import get_tsla_option_list
from czsc.analyze import CZSC


class TslaOptionSignal:
    def __init__(self) -> None:
        self.tsla_price = 0.0

    def process(self, czsc: CZSC, new_bar: bool):
        last_bar = czsc.bars_raw[-1]
        if czsc.symbol == "TSLA":
            self.tsla_price = last_bar.close
            contracts = asyncio.run(get_tsla_option_list())
            logger.debug(f"contracts: {contracts[:5]}")

    def reset(self):
        ...
