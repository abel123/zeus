import asyncio
from math import fabs

from loguru import logger
from backend.calculate.protocol import Processor
from backend.utils.convert import local_time
from backend.utils.notify import Notify
from czsc.analyze import CZSC
from talipp.indicators import EMA, SMA
from czsc.enum import Freq

from czsc.objects import Signal


class MAHit(Processor):
    def __init__(self, period=120) -> None:
        super().__init__()
        self.sma = SMA(period=period)

    def id(self) -> str:
        return f"{__name__}"

    def process(self, czsc: CZSC, new_bar: bool):
        last_bar = czsc.bars_raw[-1]

        if new_bar:
            self.sma.add_input_value(last_bar.close)
        else:
            self.sma.update_input_value(last_bar.close)
        if len(self.sma) > self.sma.period:
            self.sma.purge_oldest(1)

        k1 = "触及均线"
        k2 = "symbol"
        k3 = "freq"
        if len(self.sma) > 0:
            if fabs(last_bar.close - self.sma[-1]) / self.sma[-1] < 0.001:
                logger.warning(
                    {
                        "dt": last_bar.dt,
                        "k1": k1,
                        "v1": self.sma.period,
                        "k2": k2,
                        "v2": last_bar.symbol,
                        "k3": last_bar.freq.value,
                        "value": self.sma[-1] if len(self.sma) > 0 else None,
                    }
                )
                signal = Signal(
                    k1=k1,
                    v1=self.sma.period,
                    k2=k2,
                    v2=last_bar.symbol,
                    k3=k3,
                    v3=last_bar.freq.value,
                )
                asyncio.ensure_future(
                    Notify.send(
                        title=signal.key,
                        message=f"{local_time(last_bar.dt)} {signal.value}",
                        sound=True,
                    )
                )
                return Signal(
                    k1=k1,
                    v1=self.sma.period,
                    k2=k2,
                    v2=last_bar.symbol,
                    k3=k3,
                    v3=last_bar.freq.value,
                )
        if False and last_bar.freq == Freq.F60:
            logger.warning(
                {
                    "dt": last_bar.dt,
                    "v1": self.sma.period,
                    "k2": k2,
                    "v2": last_bar.symbol,
                    "k3": last_bar.freq.value,
                    "value": self.sma[-1] if len(self.sma) > 0 else None,
                }
            )
        return None

    def reset(self):
        self.sma.remove_all()
