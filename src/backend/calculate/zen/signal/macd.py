import asyncio
from collections import OrderedDict
from dataclasses import dataclass
from datetime import datetime, timedelta
from enum import Enum
import math
from typing import Any, List
from desktop_notifier import Urgency
from loguru import logger
from pydantic import BaseModel
from backend.datafeed.tv_model import MacdConfig
from backend.utils.convert import local_time
from backend.utils.notify import Notify
from czsc.analyze import CZSC
from czsc.enum import Direction, Freq
from czsc.objects import BI, ZS
from czsc.signals.tas import update_macd_cache
from czsc.utils.sig import create_single_signal, get_sub_elements


class Config(BaseModel):
    """
    :param kwargs: 其他参数

            - di: 倒数第 di 根 K 线
            - th: 背驰段的相应macd面积之和 <= 进入中枢段的相应面积之和 * th / 100
    """

    macd_config: List[MacdConfig]
    di: int = 1
    th: int = 90
class BCType(Enum):
    AREA_WITH_DIFF="area_with_diff"
    AREA = "area"

class MACDArea:
    class RecordCache(OrderedDict):
        def __init__(self, enable_notify=True):
            OrderedDict.__init__(self)
            self.bar_beichi = set()
            self.enable_notify = enable_notify

        def __setitem__(self, __key: Any, __value: Any) -> None:
            super().__setitem__(__key, __value)
            try:
                bc: MACDArea.BC = __value
                if self.enable_notify:
                    if bc.bi_a.raw_bars[-1].freq in [Freq.F1, Freq.F3, Freq.F5, Freq.F10, Freq.F15] and \
                        bc.bi_b.edt.timestamp() < datetime.now().timestamp()-timedelta(minutes=10).total_seconds():
                        return
                    asyncio.ensure_future(
                        Notify.send(
                        title=bc.bi_a.fx_a.raw_bars[-1].symbol+" "+ str(bc.bi_a.fx_a.raw_bars[-1].freq)+" "+ ("顶" if bc.direction == Direction.Up else "底") + "背驰",
                        message=f"{local_time(bc.bi_a.sdt).strftime("%Y-%m-%d %H:%M:%S")} - {local_time(bc.bi_b.sdt).strftime("%Y-%m-%d %H:%M:%S")}",
                        sound=True,
                        thread="背驰",
                        urgency=Urgency.Critical
                    )
                )
            except Exception as e:
                logger.exception(f"notify err {e}")
                ...
            return None

        ...


    @dataclass
    class BC:
        bi_a: BI
        bi_b: BI
        area_a: float
        area_b: float
        macd_a_dt: datetime
        macd_a_val: float
        macd_b_dt: datetime
        macd_b_val: float
        zs: ZS
        direction: Direction
        type: BCType

        @property
        def high(self):
            return max(self.bi_a.high, self.bi_b.high)

        @property
        def low(self):
            return min(self.bi_a.low, self.bi_b.low)

    def __init__(self, config: Config,enable_notify = True) -> None:
        self.config = config
        self.bc_set = OrderedDict()
        for c in config.macd_config:
            self.bc_set[c] = MACDArea.RecordCache(enable_notify)

    def add_key(self, config: List[MacdConfig]) -> None:
        for c in config:
            if c not in self.bc_set:
                self.bc_set[c] = MACDArea.RecordCache()

    def macd_area_bc(self, c: CZSC, has_new_bar: bool, **kwargs):
        for key in self.bc_set.keys():
            try:
                self.macd_area_bc_single(key, c, key, has_new_bar)
            except Exception as e:
                logger.exception(e)
                ...

    def on_bi_break(self, bi: BI):
        for key in self.bc_set.keys():
            bc_set = self.bc_set[key]
            for st1, st2 in bc_set.keys():
                if st2 == bi.sdt:
                    # logger.debug(f"remove {(st1, st2)}")
                    self.bc_set[key].pop((st1, st2))
        if False and bi.fx_b.raw_bars[-1].freq in [Freq.F1, Freq.F3, Freq.F5]:
            asyncio.ensure_future(
                Notify.send(f"{bi.fx_b.raw_bars[-1].freq} 破坏", 
                                                       message=f"{bi.fx_b.raw_bars[-1].dt.strftime("%Y-%m-%d %H:%M:%S")}"))

    def on_bi_create(self, bi: BI):
        if False and bi.fx_b.raw_bars[-1].freq in [Freq.F1, Freq.F3, Freq.F5]:
            asyncio.ensure_future(
                Notify.send(f"{bi.fx_b.raw_bars[-1].freq} 生成", 
                                                       message=f"{bi.fx_b.raw_bars[-1].dt.strftime("%Y-%m-%d %H:%M:%S")}"))

    def macd_area_bc_single(
        self, index: MacdConfig, c: CZSC, config: MacdConfig, has_new_bar: bool
    ):
        """MACD面积背驰

        参数模板："{freq}_D{di}T{th}MACD面积背驰_BS辅助V230422"

        **信号逻辑：**

        以上涨背驰为例，反之为下跌背驰：

        1. 背驰段的相应macd面积之和 <= 进入中枢段的相应面积之和 * th / 100
        2. 中枢把黄白线拉到0轴附近，
        3. 离开中枢的一笔，黄白线大于0且不新高

        **信号列表：**

        - Signal('15分钟_D1T50MACD面积背驰_BS辅助_上涨_9笔_任意_0')
        - Signal('15分钟_D1T50MACD面积背驰_BS辅助_上涨_7笔_任意_0')

        :param c: 基础周期的 CZSC 对象

        :return: 信号字典
        """
        if config.source == "volume":
            return
        if len(c.bars_raw) >= 2:
            cache_key = update_macd_cache(
                c,
                fastperiod=config.fast,
                slowperiod=config.slow,
                signalperiod=config.signal,
            )

        di, th = self.config.di, self.config.th
        freq = c.freq.value
        k1, k2, k3 = f"{freq}_D{di}T{th}MACD面积背驰_BS辅助".split("_")
        v1 = "其他"
        if len(c.bi_list) < 7 or len(c.bars_ubi) > 7:
            return create_single_signal(k1=k1, k2=k2, k3=k3, v1=v1)

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

            min_low = min(x.low for x in bis)
            max_high = max(x.high for x in bis)
            type=BCType.AREA

            if (
                bi1.direction == Direction.Up
                and bi1.low == min_low
                and bi2.high == max_high
                and (dif_zero <= 0.00001 or math.fabs(dif_zero) < math.fabs(bi1_dif)/4)
                and (bi1_dif > 0 and bi2_dif >0 )
            ):
                macd_a = max(
                    [
                        bar
                        for bar in bi1.raw_bars
                        if not math.isnan(bar.cache[cache_key]["macd"])
                    ],
                    key=lambda bar: bar.cache[cache_key]["macd"],
                )
                macd_b = max(
                    bi2.raw_bars, key=lambda bar: bar.cache[cache_key]["macd"]
                ) if len(bi2.raw_bars)>0 else bi2.raw_bars[-1]

                if bi1_dif > bi2_dif > 0:
                    type = BCType.AREA_WITH_DIFF

                if self.bc_set[index].get((bi1.sdt, bi2.sdt)) == None:
                    #logger.debug(f"upsert {bi1.sdt}-{bi2.sdt}")
                    self.bc_set[index][(bi1.sdt, bi2.sdt)] = MACDArea.BC(
                    bi_a=bi1,
                    bi_b=bi2,
                    macd_a_dt=macd_a.dt,
                    macd_a_val=0.0 if math.isnan(macd_a.cache[cache_key]["macd"]) else macd_a.cache[cache_key]["macd"],
                    macd_b_dt=macd_b.dt,
                    macd_b_val=macd_b.cache[cache_key]["macd"],
                    zs=zs,
                    area_a=bi1_area,
                    area_b=bi2_area,
                    type=type,
                    direction=bi1.direction,
                )

                return create_single_signal(k1=k1, k2=k2, k3=k3, v1="上涨", v2=f"{n}笔")

            if (
                bi1.direction == Direction.Down
                and bi1.high == max_high
                and bi2.low == min_low
                and (dif_zero >= -0.00001 or math.fabs(dif_zero) < math.fabs(bi1_dif)/4)
                and (bi1_dif < 0 and bi2_dif < 0)
            ):
                macd_a = min(
                    bi1.raw_bars, key=lambda bar: bar.cache[cache_key]["macd"]
                )
                macd_b = min(
                    bi2.raw_bars, key=lambda bar: bar.cache[cache_key]["macd"]
                ) if len(bi2.raw_bars)>0 else bi2.raw_bars[-1]
                if bi1_dif < bi2_dif < 0:
                    type = BCType.AREA_WITH_DIFF

                if self.bc_set[index].get((bi1.sdt, bi2.sdt)) == None:
                    self.bc_set[index][(bi1.sdt, bi2.sdt)] = MACDArea.BC(
                    bi_a=bi1,
                    bi_b=bi2,
                    macd_a_dt=macd_a.dt,
                    macd_a_val=0.0 if math.isnan(macd_a.cache[cache_key]["macd"]) else macd_a.cache[cache_key]["macd"],
                    macd_b_dt=macd_b.dt,
                    macd_b_val=macd_b.cache[cache_key]["macd"],
                    zs=zs,
                    area_a=bi1_area,
                    type=type,
                    area_b=bi2_area,
                    direction=bi1.direction,
                )
                # logger.debug(f"beichi {self.bc_set[index][(bi1.sdt, bi2.sdt)]}")
                return create_single_signal(k1=k1, k2=k2, k3=k3, v1="下跌", v2=f"{n}笔")

        return create_single_signal(k1=k1, k2=k2, k3=k3, v1=v1)

    @property
    def bc_records(self):
        return self.bc_set
