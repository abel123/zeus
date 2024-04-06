from datetime import datetime
from typing import Dict, List, Tuple

from ib_insync import BarData, BarDataList
from loguru import logger
from backend.broker.ib.subscribe_manager import WatcherProtocol
from backend.calculate.protocol import Symbol, SymbolType
from backend.calculate.zen import signal
from backend.calculate.zen.signal.macd import MACDArea
from backend.datafeed.tv_model import MacdConfig
from backend.utils.model_convert import to_czsc_bar
from czsc.analyze import CZSC
from czsc.enum import Freq
from czsc.objects import RawBar


class Watcher(WatcherProtocol):
    def __init__(
        self,
        bars: List[BarData],
        symbol: Symbol,
        freq: Freq,
        macd_config=signal.macd.Config(
            macd_config=[MacdConfig(fast=12, slow=26, signal=9)]
        ),
        reset=True,
    ) -> None:
        self.macd_config = macd_config
        self.symbol = symbol
        self.freq: Freq = freq
        if reset:
            self.reset(self.symbol.raw, self.freq)
        for bar in bars:
            self.on_bar_update(bar, True)

    def id(self) -> str:
        return f"{__name__}-{self.symbol.raw}:{self.freq}"

    def contracts(self) -> List[Tuple[Symbol, Freq]]:
        return [(self.symbol, self.freq)]

    def on_bar_update(self, bar: RawBar, hasNewBar):
        if isinstance(bar, BarData):
            bar = to_czsc_bar(self.symbol, self.freq, bar)
        self.czsc.update(bar)
        self.macd_signal.macd_area_bc(self.czsc, hasNewBar)

    def reset(self, symbol_raw: str, freq: Freq):
        logger.warning(f"reset {self.symbol} - {self.freq}")

        self.macd_signal = MACDArea(self.macd_config)
        self.czsc: CZSC = CZSC(
            self.symbol,
            self.freq,
            bars=[],
            get_signals=None,
            on_bi_break=self.macd_signal.on_bi_break,
            on_bi_create=self.macd_signal.on_bi_create,
        )
