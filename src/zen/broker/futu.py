import asyncio
from datetime import date, datetime, timedelta, timezone
from enum import Enum
import math
import os
from ib_insync import *
import zen_core
from broker.enums import Resolution
from cachetools import TTLCache

import flag

from moomoo.quote.quote_query import *
from loguru import logger

from moomoo_async.quoto import Quoto


class Listener:
    mapping_ = {
        KLType.K_1M: zen_core.Freq.F1,
        KLType.K_3M: zen_core.Freq.F3,
        KLType.K_5M: zen_core.Freq.F5,
        KLType.K_15M: zen_core.Freq.F15,
        KLType.K_60M: zen_core.Freq.F60,
        KLType.K_DAY: zen_core.Freq.D,
        KLType.K_WEEK: zen_core.Freq.W,
    }

    def __init__(self, futu: Quoto, bars: BarDataList):
        self.futu = futu
        self.bars = bars
        self.zen = zen_core.Zen(
            str(bars.contract.symbol),
            Listener.mapping_.get(bars.barSizeSetting, zen_core.Freq.F60),
        )
        for bar in bars:
            self.zen.append(
                zen_core.Bar(
                    bar.date,
                    bar.open,
                    bar.close,
                    bar.high,
                    bar.low,
                    bar.volume,
                ),
                False,
            )
        logger.debug(
            "bars {} {} {}", bars.contract.symbol, bars.barSizeSetting, len(bars)
        )
        self.bars.updateEvent += self._update_data

    def __setstate__(self, state):
        # logger.debug("state {}", state)
        self.bars = state["bars"]
        self.zen = zen_core.Zen(
            str(self.bars.contract.symbol),
            Listener.mapping_.get(self.bars.barSizeSetting, zen_core.Freq.F60),
        )
        for bar in self.bars:
            self.zen.append(
                zen_core.Bar(
                    bar.date,
                    bar.open,
                    bar.close,
                    bar.high,
                    bar.low,
                    bar.volume,
                ),
                False,
            )
        # logger.debug("bars {}", len(self.bars))

    def _update_data(self, bars: BarDataList, hasNewBar):
        bar = bars[-1]
        # logger.debug("{} {} {}", bars.barSizeSetting, bar.date, bar.close)
        self.zen.append(
            zen_core.Bar(
                bar.date,
                bar.open,
                bar.close,
                bar.high,
                bar.low,
                bar.volume,
            ),
            True,
        )

    def __del__(self):
        if self.bars.keepUpToDate:
            kline = {
                KLType.K_1M: SubType.K_1M,
                KLType.K_3M: SubType.K_3M,
                KLType.K_5M: SubType.K_5M,
                KLType.K_15M: SubType.K_15M,
                KLType.K_60M: SubType.K_60M,
                KLType.K_DAY: SubType.K_DAY,
                KLType.K_WEEK: SubType.K_WEEK,
            }.get(self.bars.barSizeSetting, None)

            logger.debug(
                "remove item({}) {} - {}:{} : {} - {}",
                self.bars.reqId,
                self.bars.contract,
                self.bars.barSizeSetting,
                kline,
                self.bars[0].date if len(self.bars) > 0 else None,
                self.bars[-1].date if len(self.bars) > 0 else None,
            )
            asyncio.create_task(
                self.futu.unsubscribe([self.bars.contract.symbol], [kline])
            )


class Broker:
    mapping: dict[Resolution, SubType] = {
        Resolution.Min: SubType.K_1M,
        Resolution.Min3: SubType.K_3M,
        Resolution.Min5: SubType.K_5M,
        Resolution.Min15: SubType.K_15M,
        Resolution.Min30: SubType.K_30M,
        Resolution.Hour: SubType.K_60M,
        Resolution.Day: SubType.K_DAY,
        Resolution.Week: SubType.K_WEEK,
        Resolution.Month: SubType.K_MON,
    }

    def __init__(self):
        self.futu = Quoto()
        self.cache_key = dict()
        self.cache = TTLCache(maxsize=90, ttl=timedelta(hours=1).seconds)

    def __del__(self):
        self.cache.clear()

    async def _subscribe(
        self,
        contract: Contract,
        count: int,
        sub_type: SubType,
        realtime: bool,
    ) -> Listener:
        ok, msg, _ = await self.futu.subscribe(
            [contract.symbol], [sub_type], is_first_push=True, subscribe_push=realtime
        )
        logger.debug("subscribe {} {}", ok, msg)
        kline = {
            SubType.K_1M: KLType.K_1M,
            SubType.K_3M: KLType.K_3M,
            SubType.K_5M: KLType.K_5M,
            SubType.K_15M: KLType.K_15M,
            SubType.K_60M: KLType.K_60M,
            SubType.K_DAY: KLType.K_DAY,
            SubType.K_WEEK: KLType.K_WEEK,
        }.get(sub_type, None)
        ok, msg, bars = await self.futu.get_cur_kline(
            contract.symbol, count, ktype=kline, keep_update=realtime
        )
        logger.debug("get_cur_kline {} {}", ok, msg)
        return Listener(self.futu, bars)

    async def subscribe(
        self,
        symbol: str,
        freq: Resolution,
        from_: int,
        to: int,
        non_realtime: bool,
    ):
        realtime = non_realtime == False and datetime.fromtimestamp(
            to
        ) > datetime.now() - timedelta(days=2 * 365)

        key = f"{symbol}:{freq}:{realtime}"
        if realtime:
            to = datetime.now().timestamp()
            l: Listener = self.cache.get(key)
            if l != None:
                bar = l.bars[0]
                first = bar.date

                if first.timestamp() <= from_:
                    return l
                else:
                    logger.debug(
                        "cache expire {} - {}  | {} - {}",
                        first,
                        datetime.fromtimestamp(
                            from_,
                            tz=timezone(timedelta(hours=-4), "EST"),
                        ),
                        first.timestamp(),
                        from_,
                    )

        start = datetime.now()
        listener = await self._subscribe(
            Stock(symbol, "SMART", "USD"),
            1000,
            self.mapping[freq],
            realtime,
        )

        if realtime:
            logger.debug(
                "subscribe time {}: {} - {}",
                datetime.now() - start,
                listener.bars[0].date if len(listener.bars) > 0 else None,
                listener.bars[-1].date if len(listener.bars) > 0 else None,
            )
            self.cache[key] = listener
            self.cache_key[listener.bars.reqId] = key

        return listener
