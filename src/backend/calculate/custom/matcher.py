from datetime import datetime
from typing import List

from loguru import logger
from backend.calculate.protocol import Symbol
from czsc.enum import Operate

from czsc.objects import Event, Factor, Signal


class Matcher:
    def __init__(self, events: List[Event]) -> None:
        self.events: List[Event] = events

    def match(self, signals: List[Signal], dt: datetime, symbol: str = "default"):
        ss = dict()
        for s in signals:
            ss[s.key] = s.value

        factors = []
        for s in self.events:
            matched, f = s.is_match(ss, dt, symbol)
            if matched:
                factors.append(f)
                logger.warning(f"factors {factors}")


DefaultMatcher = Matcher(
    [
        Event(
            operate=Operate.LO,
            factors=[
                Factor(
                    name="底背驰[小待确认、大推导]",
                    signals_all=[],
                    signals_any=[
                        Signal("3分钟_D1T90MACD-4-9-9-面积背驰_BS推笔辅助_底_3笔_任意_80"),
                        Signal("3分钟_D1T90MACD-12-26-9-面积背驰_BS推笔辅助_底_3笔_任意_80"),
                        Signal("3分钟_D1T90MACD-4-9-9-面积背驰_BS推笔辅助_底_5笔_任意_80"),
                        Signal("3分钟_D1T90MACD-12-26-9-面积背驰_BS推笔辅助_底_5笔_任意_80"),
                        Signal("3分钟_D1T90MACD-4-9-9-面积背驰_BS推笔辅助_底_7笔_任意_80"),
                        Signal("3分钟_D1T90MACD-12-26-9-面积背驰_BS推笔辅助_底_7笔_任意_80"),
                    ],
                    enable_notify=True,
                ),
            ],
            signals_any=[
                Signal("1分钟_D1T90MACD-4-9-9-面积背驰_BS辅助_底_3笔_任意_80"),
                Signal("1分钟_D1T90MACD-4-9-9-面积背驰_BS辅助_底_5笔_任意_80"),
                Signal("1分钟_D1T90MACD-4-9-9-面积背驰_BS辅助_底_7笔_任意_80"),
            ],
        ),
        Event(
            operate=Operate.LO,
            factors=[
                Factor(
                    name="顶背驰[小待确认、大推导]",
                    signals_all=[],
                    signals_any=[
                        Signal("3分钟_D1T90MACD-4-9-9-面积背驰_BS推笔辅助_顶_3笔_任意_80"),
                        Signal("3分钟_D1T90MACD-12-26-9-面积背驰_BS推笔辅助_顶_3笔_任意_80"),
                        Signal("3分钟_D1T90MACD-4-9-9-面积背驰_BS推笔辅助_顶_5笔_任意_80"),
                        Signal("3分钟_D1T90MACD-12-26-9-面积背驰_BS推笔辅助_顶_5笔_任意_80"),
                        Signal("3分钟_D1T90MACD-4-9-9-面积背驰_BS推笔辅助_顶_7笔_任意_80"),
                        Signal("3分钟_D1T90MACD-12-26-9-面积背驰_BS推笔辅助_顶_7笔_任意_80"),
                        Signal("5分钟_D1T90MACD-4-9-9-面积背驰_BS推笔辅助_顶_3笔_任意_80"),
                        Signal("5分钟_D1T90MACD-12-26-9-面积背驰_BS推笔辅助_顶_3笔_任意_80"),
                        Signal("5分钟_D1T90MACD-4-9-9-面积背驰_BS推笔辅助_顶_5笔_任意_80"),
                        Signal("5分钟_D1T90MACD-12-26-9-面积背驰_BS推笔辅助_顶_5笔_任意_80"),
                        Signal("5分钟_D1T90MACD-4-9-9-面积背驰_BS推笔辅助_顶_7笔_任意_80"),
                        Signal("5分钟_D1T90MACD-12-26-9-面积背驰_BS推笔辅助_顶_7笔_任意_80"),
                    ],
                    enable_notify=True,
                ),
            ],
            signals_any=[
                Signal("1分钟_D1T90MACD-4-9-9-面积背驰_BS辅助_顶_3笔_任意_80"),
                Signal("1分钟_D1T90MACD-4-9-9-面积背驰_BS辅助_顶_5笔_任意_80"),
                Signal("1分钟_D1T90MACD-4-9-9-面积背驰_BS辅助_顶_7笔_任意_80"),
            ],
        ),
        Event(
            operate=Operate.LO,
            factors=[
                Factor(
                    name="顶背驰 - 分型待确认",
                    signals_all=[],
                    signals_any=[
                        Signal("1分钟_D1T90MACD-4-9-9-面积背驰_BS辅助_顶_3笔_任意_80"),
                        Signal("1分钟_D1T90MACD-4-9-9-面积背驰_BS辅助_顶_5笔_任意_80"),
                        Signal("1分钟_D1T90MACD-4-9-9-面积背驰_BS辅助_顶_7笔_任意_80"),
                        Signal("1分钟_D1T90MACD-4-9-9-面积背驰_BS辅助_顶_9笔_任意_80"),
                    ],
                    enable_notify=True,
                ),
            ],
            signals_all=[
                Signal("分型_1分钟_任意_顶_1分钟_任意_0"),
            ],
        ),
        Event(
            operate=Operate.LO,
            factors=[
                Factor(
                    name="底背驰 - 分型待确认",
                    signals_all=[],
                    signals_any=[
                        Signal("1分钟_D1T90MACD-4-9-9-面积背驰_BS辅助_底_3笔_任意_80"),
                        Signal("1分钟_D1T90MACD-4-9-9-面积背驰_BS辅助_底_5笔_任意_80"),
                        Signal("1分钟_D1T90MACD-4-9-9-面积背驰_BS辅助_底_7笔_任意_80"),
                        Signal("1分钟_D1T90MACD-4-9-9-面积背驰_BS辅助_底_9笔_任意_80"),
                    ],
                    enable_notify=True,
                ),
            ],
            signals_all=[
                Signal("分型_1分钟_任意_底_1分钟_任意_0"),
            ],
        ),
    ]
)
