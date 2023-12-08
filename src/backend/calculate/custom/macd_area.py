from enum import Enum
import math
from typing import List

from pydantic import BaseModel
from backend.calculate.protocol import Processor
from backend.datafeed.tv_model import MacdConfig
from backend.utils.convert import local_time
from czsc.analyze import CZSC
from czsc.enum import Direction
from czsc.objects import ZS, Signal
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
            evs.append(self.macd_area_bc_single(czsc, c))
        return evs

    def macd_area_bc_single(self, c: CZSC, config: MacdConfig):
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
        ) = f"{freq}_D{di}T{th}MACD-{config.fast}-{config.slow}-{config.signal}-面积背驰_BS辅助".split(
            "_"
        )
        v1 = "其他"
        if len(c.bi_list) < 7 or len(c.bars_ubi) > 7:
            return Signal(k1=k1, k2=k2, k3=k3, v1=v1)

        for n in (9, 7, 5, 3):
            bis = get_sub_elements(c.bi_list, di=di, n=n)
            if len(bis) != n:
                continue

            # 假定离开中枢的都是一笔
            zs = ZS(bis[1:-1])
            if not zs.is_valid or len(bis[-1].raw_bars) < 1:  # 如果中枢不成立，往下进行
                continue

            bi1, bi2 = bis[0], bis[-1]
            bi1_macd = [x.cache[cache_key]["macd"] for x in bi1.raw_bars]
            bi2_macd = [x.cache[cache_key]["macd"] for x in bi2.raw_bars]

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

            min_low = min(x.low for x in bis)
            max_high = max(x.high for x in bis)
            score = 80
            if (
                bi1.direction == Direction.Up
                and bi1.low == min_low
                and bi2.high == max_high
                and (
                    dif_zero <= 0.00001 or math.fabs(dif_zero) < math.fabs(bi1_dif) / 4
                )
                and (bi1_dif > 0 and bi2_dif > 0)
            ):
                if bi1_dif > bi2_dif > 0:
                    score = 100

                return Signal(k1=k1, k2=k2, k3=k3, v1="顶", v2=f"{n}笔", score=score)

            if (
                bi1.direction == Direction.Down
                and bi1.high == max_high
                and bi2.low == min_low
                and (
                    dif_zero >= -0.00001 or math.fabs(dif_zero) < math.fabs(bi1_dif) / 4
                )
                and (bi1_dif < 0 and bi2_dif < 0)
            ):
                if bi1_dif < bi2_dif < 0:
                    score = 100

                return Signal(k1=k1, k2=k2, k3=k3, v1="底", v2=f"{n}笔", score=score)

        return Signal(k1=k1, k2=k2, k3=k3, v1=v1)

    def reset(self):
        ...
