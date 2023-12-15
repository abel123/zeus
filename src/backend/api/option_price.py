from typing import Dict, List, Tuple
from sanic import Request
from backend.calculate.protocol import Symbol, WatcherProtocol

from backend.utils.magic import SingletonABCMeta
from czsc.analyze import CZSC
from czsc.enum import Freq


class Tracker:
    def __init__(self, symbol: str, period: int) -> None:
        self.symbol = symbol
        self.period = period
        self.ma_price = None
        self.put_price = None
        self.call_price = None


class OptionTracker(WatcherProtocol, metaclass=SingletonABCMeta):
    def __init__(
        self, symbol_list: List[Symbol], ma_periods: List[int] = [120, 200]
    ) -> None:
        self.symbol_list = symbol_list
        self.trackers: Dict[Tuple[str, Freq], Tracker] = dict()
        for symbol in symbol_list:
            for p in ma_periods:
                self.trackers[(symbol, p)] = Tracker(symbol, p)

    def id(self):
        return f"{__name__}_{self.__class__.__name__}"

    def option_price(self, request: Request):
        ...

    def process(self, czsc: CZSC, new_bar: bool):
        ...

    def reset(self):
        ...
