from datetime import datetime
from typing import List

from loguru import logger
from czsc.enum import Operate

from czsc.objects import Event, Factor, Signal


class Matcher:
    def __init__(self, events: List[Event]) -> None:
        self.events: List[Event] = events

    def match(self, signals: List[Signal], dt: datetime):
        ss = dict()
        for s in signals:
            ss[s.key] = s.value

        factors = []
        for s in self.events:
            matched, f = s.is_match(ss, dt)
            if matched:
                factors.append(f)
                logger.warning(f"factors {factors}")


DefaultMatcher = Matcher(
    [
        Event(
            operate=Operate.LO,
            factors=[
                Factor(
                    name="背驰 - 分型待确认",
                    signals_all=[
                        Signal("1分钟_D1T90MACD-4-9-9-面积背驰_BS辅助_顶_3笔_任意_80"),
                        Signal("分型_freq_任意_顶_1分钟_任意_0"),
                    ],
                    enable_notify=True,
                )
            ],
        )
    ]
)
