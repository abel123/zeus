from czsc.analyze import CZSC, check_fxs
from czsc.enum import Mark
from czsc.objects import Signal


class FxCheck:
    def __init__(self) -> None:
        pass

    def process(self, czsc: CZSC, new_bar: bool):
        if len(czsc.bars_ubi) != 3:
            return
        fxs = check_fxs(czsc.bars_ubi)
        if len(fxs) == 0:
            return

        k1 = "分型"
        k2 = czsc.freq.value

        if fxs[0].mark == Mark.D:
            return Signal(
                k1=k1,
                v1="底",
                k2=k2,
                v2=czsc.bars_raw[-1].freq.value,
            )
        else:
            return Signal(
                k1=k1,
                v1="顶",
                k2=k2,
                v2=czsc.bars_raw[-1].freq.value,
            )

    def reset(self):
        ...
