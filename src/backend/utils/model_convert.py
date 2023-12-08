from datetime import datetime
from ib_insync import BarData
import pytz
from backend.calculate.protocol import Symbol
from czsc.enum import Freq

from czsc.objects import RawBar


def to_czsc_bar(symbol: Symbol, freq: Freq, bar: BarData):
    return RawBar(
        symbol=symbol.raw,
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
