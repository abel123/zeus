from enum import Enum
import math
from typing import List
from loguru import logger

from pydantic import BaseModel
from backend.calculate.protocol import Processor
from backend.datafeed.tv_model import MacdConfig
from backend.utils.convert import local_time
from czsc.analyze import CZSC
from czsc.enum import Direction, Freq
from czsc.objects import ZS, RawBar, Signal
from czsc.signals.tas import update_macd_cache
from czsc.utils.sig import create_single_signal, get_sub_elements


class BCType(Enum):
    AREA_WITH_DIFF = "area_with_diff"
    AREA = "area"


class Config(BaseModel):
    """
    :param kwargs: 其他参数

            - di: 倒数第 di 根 K 线
            - th: 背驰段的相应macd面积之和 <= 进入中枢段的相应面积之和 * th / 100
    """

    macd_config: List[MacdConfig]
    di: int = 1
    th: int = 90


class Fake_BI(BaseModel):
    raw_bars: List[RawBar]
    low: float
    high: float


class MACDArea(Processor):
    def __init__(
        self,
        config: Config = Config(
            macd_config=[
                MacdConfig(fast=4, slow=9, signal=9),
                MacdConfig(fast=12, slow=26, signal=9),
            ]
        ),
    ) -> None:
        self.config = config

    def process(self, czsc: CZSC, new_bar: bool):
        evs = []
        for c in self.config.macd_config:
            for fake in [False, True]:
                ev = self.macd_area_bc_single(czsc, c, fake)
                if isinstance(ev, Signal):
                    evs.append(ev)
                elif ev is not None:
                    evs.extend(ev)
        return evs

    def macd_area_bc_single(self, c: CZSC, config: MacdConfig, fake: bool):
        """MACD面积背驰

        参数模板："{freq}_D{di}T{th}MACD面积背驰_BS辅助V1"

        **信号逻辑：**

        以上涨背驰为例，反之为下跌背驰：

        1. 背驰段的相应macd面积之和 <= 进入中枢段的相应面积之和 * th / 100
        2. 中枢把黄白线拉到0轴附近
        3. 离开中枢的一笔, 黄白线大于0(且不新高)

        **信号列表：**

        - Signal('15分钟_D1T50MACD面积背驰_BS辅助_上涨_9笔_任意_0')
        - Signal('15分钟_D1T50MACD面积背驰_BS辅助_上涨_7笔_任意_0')

        :param c: 基础周期的 CZSC 对象

        :return: 信号字典
        """

        if fake and c.freq == Freq.F1:
            return None

        events = []
        if len(c.bars_raw) >= 2:
            cache_key = update_macd_cache(
                c,
                fastperiod=config.fast,
                slowperiod=config.slow,
                signalperiod=config.signal,
                source=config.source,
            )

        di, th = self.config.di, self.config.th
        freq = c.freq.value
        (
            k1,
            k2,
            k3,
        ) = f"{freq}_D{di}T{th}MACD-{config.fast}-{config.slow}-{config.signal}-面积背驰_BS{'推笔' if fake else''}辅助".split(
            "_"
        )
        v1 = "其他"
        if len(c.bi_list) < 7:
            return None
        if fake and len(c.bars_ubi[1:]) < 4:
            return None
        if fake == False and len(c.bars_ubi[1:]) > 2:
            return None

        offset = 1 if fake else 0
        for n in (9, 7, 5, 3):
            bis = get_sub_elements(c.bi_list, di=di, n=n - offset)
            if len(bis) != n - offset:
                continue

            # 假定离开中枢的都是一笔
            zs = ZS(bis[1 : (-1 if fake == False else None)])
            if fake:
                # logger.warning(f"zs {zs} {zs.is_valid}")
                ...
            if not zs.is_valid:  # 如果中枢不成立，往下进行
                continue

            bi1 = bis[0]
            bi2 = (
                bis[-1]
                if fake == False
                else Fake_BI(
                    raw_bars=[y for x in c.bars_ubi[1:] for y in x.raw_bars],
                    high=0.0,
                    low=0.0,
                )
            )
            if fake:
                bi2.high = max(bi2.raw_bars[0].high, bi2.raw_bars[-1].high)
                bi2.low = min(bi2.raw_bars[0].low, bi2.raw_bars[-1].low)
                inside_high = max(bi2.raw_bars, key=lambda bar: bar.high)
                if bi2.high != inside_high.high:
                    continue
                inside_low = min(bi2.raw_bars, key=lambda bar: bar.low)
                if bi2.low != inside_low.low:
                    continue

                # logger.warning(f"fake bi {bi2}")
            bi1_macd = [x.cache[cache_key]["macd"] for x in bi1.raw_bars]
            bi2_macd = [x.cache[cache_key]["macd"] for x in bi2.raw_bars]
            if len(bi1.raw_bars) < 1 or len(bi2.raw_bars) < 1:
                return None
            bi1_dif = bi1.raw_bars[-1].cache[cache_key]["dif"]
            bi2_dif = bi2.raw_bars[-1].cache[cache_key]["dif"]

            zs_fxb_raw = [y for x in zs.bis for y in x.fx_b.raw_bars]

            if bi1.direction == Direction.Up:
                bi1_area = sum([x for x in bi1_macd if x > 0])
                bi2_area = sum([x for x in bi2_macd if x > 0])
                dif_zero = min([x.cache[cache_key]["dif"] for x in zs_fxb_raw])
            else:
                bi1_area = sum([x for x in bi1_macd if x < 0])
                bi2_area = sum([x for x in bi2_macd if x < 0])
                dif_zero = max([x.cache[cache_key]["dif"] for x in zs_fxb_raw])

            if abs(bi2_area) > abs(bi1_area) * th / 100:  # 如果面积背驰不成立，往下进行
                continue

            min_low = min(min(x.low for x in bis), bi2.low)
            max_high = max(max(x.high for x in bis), bi2.high)
            score = 80
            threshold = 8.0 / 10

            if (
                bi1.direction == Direction.Up
                and bi1.low == min_low
                and bi2.high == max_high
                and (
                    dif_zero <= 0.00001
                    or math.fabs(dif_zero) < math.fabs(bi1_dif) * threshold
                )
                and (bi1_dif > 0 and bi2_dif > 0)
            ):
                if bi1_dif > bi2_dif > 0:
                    score = 100

                events.append(
                    Signal(k1=k1, k2=k2, k3=k3, v1="顶", v2=f"{n}笔", score=score)
                )

            if (
                bi1.direction == Direction.Down
                and bi1.high == max_high
                and bi2.low == min_low
                and (
                    dif_zero >= -0.00001
                    or math.fabs(dif_zero) < math.fabs(bi1_dif) * threshold
                )
                and (bi1_dif < 0 and bi2_dif < 0)
            ):
                if bi1_dif < bi2_dif < 0:
                    score = 100

                events.append(
                    Signal(k1=k1, k2=k2, k3=k3, v1="底", v2=f"{n}笔", score=score)
                )

        return events

    def reset(self): ...
