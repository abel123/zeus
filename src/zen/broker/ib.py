import asyncio
from datetime import date, datetime, timedelta, timezone
from enum import Enum
import math
import os
from ib_insync import *
from loguru import logger
import zen_core
from broker.enums import Resolution
from cachetools import TTLCache

import flag


class BarSize(Enum):
    Sec = "1 sec"
    Sec5 = "5 secs"
    Sec15 = "15 secs"
    Sec30 = "30 secs"
    Min = "1 min"
    Min2 = "2 mins"
    Min3 = "3 mins"
    Min5 = "5 mins"
    Min15 = "15 mins"
    Min20 = "20 mins"
    Min30 = "30 mins"
    Hour = "1 hour"
    Hour2 = "2 hours"
    Hour3 = "3 hours"
    Hour4 = "4 hours"
    Hour8 = "8 hours"
    Day = "1 day"
    Week = "1 week"
    Month = "1 month"


def adjust(d: date | datetime):
    return (
        datetime(
            year=d.year,
            month=d.month,
            day=d.day,
            tzinfo=timezone(timedelta(hours=-4), "EST"),
        )
        if not isinstance(d, datetime)
        else d
    )


class Listener:
    mapping_ = {
        BarSize.Min: zen_core.Freq.F1,
        BarSize.Min3: zen_core.Freq.F3,
        BarSize.Min5: zen_core.Freq.F5,
        BarSize.Min15: zen_core.Freq.F15,
        BarSize.Hour: zen_core.Freq.F60,
        BarSize.Day: zen_core.Freq.D,
        BarSize.Week: zen_core.Freq.W,
    }

    def __init__(self, ib: IB, bars: BarDataList):
        self.ib = ib
        self.bars = bars
        self.zen = zen_core.Zen(
            str(bars.contract.symbol),
            Listener.mapping_.get(BarSize(bars.barSizeSetting), zen_core.Freq.F60),
        )
        for bar in bars:
            self.zen.append(
                zen_core.Bar(
                    adjust(bar.date),
                    bar.open,
                    bar.close,
                    bar.high,
                    bar.low,
                    bar.volume,
                ),
                False,
            )
        logger.debug("bars {}", len(bars))
        self.bars.updateEvent += self._update_data

    def __setstate__(self, state):
        # logger.debug("state {}", state)
        self.bars = state["bars"]
        self.zen = zen_core.Zen(
            str(self.bars.contract.symbol),
            Listener.mapping_.get(BarSize(self.bars.barSizeSetting), zen_core.Freq.F60),
        )
        for bar in self.bars:
            self.zen.append(
                zen_core.Bar(
                    adjust(bar.date),
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
        self.zen.append(
            zen_core.Bar(
                adjust(bar.date),
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
            logger.debug(
                "remove item({}) {} - {} : {} - {}",
                self.bars.reqId,
                self.bars.contract,
                self.bars.barSizeSetting,
                self.bars[0].date if len(self.bars) > 0 else None,
                self.bars[-1].date if len(self.bars) > 0 else None,
            )
            self.ib.cancelHistoricalData(self.bars)


class Broker:
    mapping: dict[Resolution, BarSize] = {
        Resolution.Min: BarSize.Min,
        Resolution.Min3: BarSize.Min3,
        Resolution.Min5: BarSize.Min5,
        Resolution.Min15: BarSize.Min15,
        Resolution.Min30: BarSize.Min30,
        Resolution.Hour: BarSize.Hour,
        Resolution.Day: BarSize.Day,
        Resolution.Week: BarSize.Week,
        Resolution.Month: BarSize.Month,
    }

    def __init__(self):
        self.ib = IB()
        self.cache_key = dict()
        self.ib.errorEvent += self.on_error

        self.lock = asyncio.Lock()
        self.cache = TTLCache(maxsize=90, ttl=timedelta(hours=1).seconds)

    def __del__(self):
        self.cache.clear()

    async def _reconnect(self):
        async with self.lock:
            if self.ib.client.connState == Client.DISCONNECTED:
                await self.ib.connectAsync(
                    clientId=os.getenv("ID", 1), port=14001, readonly=True
                )

    async def _subscribe(
        self,
        contract: Contract,
        end_date: date | datetime | str | None,
        duration: str,
        bar_size: str,
        realtime: bool,
    ) -> Listener:
        bars: BarDataList = await self.ib.reqHistoricalDataAsync(
            contract,
            endDateTime=end_date,
            durationStr=duration,
            barSizeSetting=bar_size,
            whatToShow="TRADES",
            useRTH=flag.RTH,
            formatDate=2,
            keepUpToDate=realtime,
        )
        return Listener(self.ib, bars)

    async def subscribe(
        self,
        symbol: str,
        freq: Resolution,
        from_: int,
        to: int,
        non_realtime: bool,
    ):
        await self._reconnect()

        realtime = non_realtime == False and datetime.fromtimestamp(
            to
        ) > datetime.now() - timedelta(days=2 * 365)

        key = f"{symbol}:{freq}:{realtime}"
        if realtime:
            to = datetime.now().timestamp()
            l: Listener = self.cache.get(key)
            if l != None:
                bar = l.bars[0]
                first = adjust(bar.date)

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
            "" if realtime else datetime.fromtimestamp(to),
            to_duration(to - from_),
            self.mapping[freq].value,
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

    def on_error(self, reqId, errorCode, errorString, contract):
        if reqId in self.cache_key:
            key = self.cache_key[reqId]
            if self.cache.get(key) != None and self.cache.get(key).bars.reqId == reqId:
                logger.debug("del key {}", key)
                self.cache.pop(key, default=None)

            self.cache_key.pop(reqId)
        if reqId == -1 and errorCode in [1100, 2103, 2106]:
            logger.warning("clear cache: {} {}: {}", reqId, errorCode, errorString)
            self.cache.clear()


def to_duration(delta: int) -> str:
    duration = max(timedelta(seconds=delta), (timedelta(days=2))) + timedelta(days=2)
    if duration.days >= 360:
        return f"{math.ceil(duration / timedelta(365))} Y"
    elif duration.days >= 30:
        return f"{math.ceil(duration / timedelta(30))} M"
    elif duration.days >= 7:
        return f"{math.ceil(duration / timedelta(7))} W"
    elif duration.days >= 1:
        return f"{math.ceil(duration / timedelta(1))} D"
    else:
        return "30 S"
