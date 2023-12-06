from loguru import logger
from backend.calculate.protocol import Processor
from czsc.analyze import CZSC
from czsc.objects import Signal


class QPReverseSignals(Processor):
    def __init__(self) -> None:
        super().__init__()
        self.reset()
        self.last_bar_len = 0

    def id(self) -> str:
        return f"{__name__}"

    def process(self, czsc: CZSC, new_bar: bool):
        try:
            new_bar = self.last_bar_len != len(czsc.bars_raw)
            self.last_bar_len = len(czsc.bars_raw)
            if new_bar and len(czsc.bars_raw) > 1:
                last_bar = czsc.bars_raw[-2]
                total = max(0.0001, last_bar.upper + last_bar.lower + last_bar.solid)
                if last_bar.lower / total > 4.0 / 10:
                    return Signal(
                        k1="symbol", v1=last_bar.symbol, k2="完结Bar形态", v2="长下影"
                    )

                elif last_bar.upper / total > 4.0 / 10:
                    return Signal(
                        k1="symbol", v1=last_bar.symbol, k2="完结Bar形态", v2="长上影"
                    )
                return Signal(k1="symbol", v1=last_bar.symbol, k2="完结Bar形态", v2="其它")

        except Exception as e:
            logger.error(e)
            return None

    def process_volume(self, czsc: CZSC, new_bar: bool):
        ...

    def reset(self):
        ...
