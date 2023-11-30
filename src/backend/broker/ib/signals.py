from datetime import datetime
import traceback
from typing import Dict, List

from ib_insync import BarData, BarDataList
from loguru import logger
from backend.broker.ib.subscribe_manager import WatcherProtocol
from backend.calculate.zen import signal
from backend.calculate.zen.signal.macd import MACDArea
from backend.datafeed.tv_model import MacdConfig
from czsc.analyze import CZSC
from czsc.enum import Freq
from czsc.objects import RawBar


class Watcher(WatcherProtocol):
    def __init__(
        self,
        bars: List[BarData],
        symbol: str,
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
            self.reset()
        for bar in bars:
            self.on_bar_update(bar, True)

    def id(self) -> str:
        return f"{__name__}-{self.symbol}:{self.freq}"

    def on_bar_update(self, bar: BarData, hasNewBar):
        self.czsc.update(
            RawBar(
                symbol=self.symbol,
                id=datetime.fromisoformat(bar.date.isoformat()).timestamp(),
                dt=datetime.fromisoformat(bar.date.isoformat()),
                freq=self.freq,
                open=bar.open,
                close=bar.close,
                high=bar.high,
                low=bar.low,
                vol=bar.volume,
                amount=0.0,
            )
        )
        self.macd_signal.macd_area_bc(self.czsc, hasNewBar)

    def reset(self):
        logger.warning(f"reset {self.symbol} - {self.freq}")

        self.macd_signal = MACDArea(self.macd_config)
        self.czsc: CZSC = CZSC(
            self.symbol,
            self.freq,
            bars=[],
            get_signals=None,  # self.macd_signal.macd_area_bc,
            on_bi_break=self.macd_signal.on_bi_break,
            on_bi_create=self.macd_signal.on_bi_create,
        )
