import asyncio
from datetime import datetime
from typing import List

from ib_insync import BarData
from loguru import logger
from backend.calculate.protocol import WatcherProtocol
from backend.utils.notify import Notify
from czsc.analyze import CZSC
from czsc.enum import Freq
from czsc.objects import RawBar


def convert(symbol, freq, bar: BarData):
    return RawBar(
        symbol=symbol,
        id=datetime.fromisoformat(bar.date.isoformat()).timestamp(),
        dt=datetime.fromisoformat(bar.date.isoformat()),
        freq=freq,
        open=bar.open,
        close=bar.close,
        high=bar.high,
        low=bar.low,
        vol=bar.volume,
        amount=0.0,
    )


class QPMReverseSignals(WatcherProtocol):
    def __init__(self, symbol: str, freq: Freq) -> None:
        super().__init__()
        self.symbol = symbol
        self.freq = freq

    def id(self) -> str:
        return f"{__name__}:{self.symbol}-{self.freq}"

    def on_bar_update(self, bar: RawBar, hasNewBar):
        try:
            if isinstance(bar, BarData):
                bar = convert(self.symbol, self.freq, bar)

            # logger.debug(
            #    f"on_bar_update {bar.symbol} {bar.freq} - {bar.dt} : {bar.upper/total} - {bar.solid/total} - {bar.lower/total}"
            # )

            self.czsc.update(bar)
            if hasNewBar and len(self.czsc.bars_raw) > 1:
                last_bar = self.czsc.bars_raw[-2]
                total = last_bar.upper + last_bar.lower + last_bar.solid

                if last_bar.lower / total > 1.0 / 3:
                    asyncio.ensure_future(
                        Notify.send(f"{self.symbol} {self.freq} 长下影", f"{last_bar.dt}")
                    )
                elif last_bar.upper / total > 1.0 / 3:
                    asyncio.ensure_future(
                        Notify.send(f"{self.symbol} {self.freq} 长上影", f"{last_bar.dt}")
                    )
        except Exception as e:
            logger.error(e)

    def reset(self):
        self.czsc: CZSC = CZSC(
            self.symbol,
            self.freq,
            bars=[],
        )

    def populate_bars(self, bars: List[RawBar]):
        for bar in bars:
            self.on_bar_update(bar, True)
