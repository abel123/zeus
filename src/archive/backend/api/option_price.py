import math
from typing import Dict, List, Tuple
from asyncache import cached
from cachetools import LRUCache
from ib_insync import Contract, Option, Ticker
from loguru import logger
from pydantic import BaseModel
from sanic import Request
from backend.broker.ib.subscribe_manager import SubscribeManager
from backend.calculate.protocol import Symbol, WatcherProtocol

from backend.utils.magic import SingletonABCMeta
from czsc.analyze import CZSC
from czsc.enum import Freq
from talipp.indicators import EMA, SMA


class Row(BaseModel):
    interval: str
    ma: int
    delta: float
    price: float
    option_price: float


class Item:
    def __init__(self, ticker: Ticker) -> None:
        self.ticker = ticker

    def __del__(self):
        logger.error(f"cancel mkt data {self.ticker.contract}")
        SubscribeManager().ib.cancelMktData(self.ticker.contract)


@cached(cache=LRUCache(maxsize=32))
async def qualifyContract(option: str):
    details = await SubscribeManager().ib.qualifyContractsAsync(
        Contract("OPT", localSymbol=option, exchange="SMART")
    )
    return details[0]


@cached(cache=LRUCache(maxsize=40))
async def get_option(option: str) -> Item:
    logger.error(f"option detail {await qualifyContract(option)}")
    ticker = SubscribeManager().ib.reqMktData(await qualifyContract(option))
    return Item(ticker)


class OptionTracker(metaclass=SingletonABCMeta):
    def __init__(
        self,
        symbol_list: List[str],
        period: List[Freq] = [Freq.F3, Freq.F5, Freq.F10, Freq.F15, Freq.F60],
        ma_interval: List[int] = [15, 30, 60, 120, 200],
    ) -> None:
        self.symbol_list = symbol_list
        self.ma_interval = ma_interval
        self.period = period
        self._reset()

    def id(self):
        return f"{__name__}_{self.__class__.__name__}"

    def process(self, czsc: CZSC, new_bar: bool):
        last_bar = czsc.bars_raw[-1]
        symbol = last_bar.symbol
        freq = last_bar.freq

        for i in self.ma_interval:
            if self.trackers.get((symbol, freq, i)) == None:
                continue
            sma = self.trackers[(symbol, freq, i)]
            if new_bar:
                sma.add_input_value(last_bar.close)
            else:
                sma.update_input_value(last_bar.close)
            if len(sma) > sma.period:
                sma.purge_oldest(1)

    async def get_option_price(self, symbol: str, option: str):
        item = await get_option(option)
        result = []
        logger.debug(
            f"======== {item.ticker.last} {item.ticker.lastGreeks} {item.ticker.modelGreeks} {item.ticker.askGreeks}"
        )
        for p in self.period:
            for ma in self.ma_interval:
                if self.trackers.get((symbol, p, ma)) == None:
                    continue
                sma = self.trackers[(symbol, p, ma)]
                if len(sma) == 0:
                    continue
                sma_price = sma[-1]
                greek = item.ticker.lastGreeks

                delta = 0.0
                if greek != None:
                    delta = greek.delta
                if delta == None:
                    delta = 0.0
                result.append(
                    Row(
                        interval=p,
                        ma=ma,
                        delta=delta,
                        price=sma_price,
                        option_price=(
                            -1.0 if math.isnan(item.ticker.last) else item.ticker.last
                        )
                        + delta * (sma_price - sma.input_values[-1]),
                    )
                )
        return result

    def _reset(self):
        self.trackers: Dict[Tuple[str, Freq, int], SMA] = dict()
        for symbol in self.symbol_list:
            for ma in self.ma_interval:
                for p in self.period:
                    self.trackers[(symbol, p, ma)] = SMA(period=ma)

    def reset(self):
        pass
