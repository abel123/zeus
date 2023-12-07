import asyncio
from loguru import logger
from backend.calculate.protocol import Processor
from backend.utils.notify import Notify
from czsc.analyze import CZSC
from czsc.objects import Signal
from talipp.indicators import SMA


class QPReverseSignals(Processor):
    def __init__(self, vl_fast=2, vl_slow=5) -> None:
        super().__init__()
        self.reset()
        self.last_bar_len = 0
        self.vl_fast = SMA(vl_fast)
        self.vl_slow = SMA(vl_slow)

    def id(self) -> str:
        return f"{__name__}"

    def process(self, czsc: CZSC, new_bar: bool):
        events = []
        ev = self.process_price(czsc, new_bar)
        ev_volume = self.process_volume(czsc, new_bar)
        if ev != None:
            events.append(ev)
        if ev_volume != None:
            events.append(ev_volume)
        return events

    def process_price(self, czsc: CZSC, new_bar: bool):
        try:
            new_bar = self.last_bar_len != len(czsc.bars_raw)
            self.last_bar_len = len(czsc.bars_raw)
            if new_bar and len(czsc.bars_raw) > 1:
                last_bar = czsc.bars_raw[-2]
                v1 = f"{last_bar.symbol}:{last_bar.freq.value}"

                total = max(0.0001, last_bar.upper + last_bar.lower + last_bar.solid)
                if last_bar.lower / total > 4.0 / 10:
                    return Signal(
                        k1="symbol",
                        v1=v1,
                        k2="完结Bar形态",
                        v2="长下影",
                    )

                elif last_bar.upper / total > 4.0 / 10:
                    return Signal(
                        k1="symbol",
                        v1=v1,
                        k2="完结Bar形态",
                        v2="长上影",
                    )
                return Signal(
                    k1="symbol",
                    v1=v1,
                    k2="完结Bar形态",
                    v2="其它",
                )

        except Exception as e:
            logger.error(e)
            return None

    def process_volume(self, czsc: CZSC, new_bar: bool):
        last_bar = czsc.bars_raw[-1]
        if new_bar:
            self.vl_fast.add_input_value(last_bar.vol)
            self.vl_slow.add_input_value(last_bar.vol)
        else:
            self.vl_fast.update_input_value(last_bar.vol)
            self.vl_slow.update_input_value(last_bar.vol)

        if len(self.vl_slow) > self.vl_slow.period:
            self.vl_slow.purge_oldest(1)
            self.vl_fast.purge_oldest(1)
        if len(self.vl_slow) > 0:
            k1, v1 = "大量", last_bar.symbol
            k2, v2 = "freq", last_bar.freq.value
            k3, v3 = "fast_slow", f"{self.vl_fast.period}-{self.vl_slow.period}"
            diff_percent = (
                (self.vl_fast[-1] - self.vl_slow[-1]) * 1.0 / self.vl_slow[-1]
            )
            if diff_percent > 0.3:
                return Signal(k1=k1, k2=k2, k3=k3, v1=v1, v2=v2, v3=v3, score=70)
        return None

    def reset(self):
        ...
