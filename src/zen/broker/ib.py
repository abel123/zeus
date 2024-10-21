import asyncio
from datetime import date, datetime, timedelta, timezone
from enum import Enum
import math
from ib_insync import *
from loguru import logger
import zen_core
from broker.enums import Resolution
from cachetools import TTLCache


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


class Listener:
    def __init__(self, ib: IB, bars: BarDataList):
        self.ib = ib
        self.bars = bars
        self.zen = zen_core.Zen(str(bars.contract), zen_core.Freq.F60)
        for bar in bars:
            self.zen.append(
                zen_core.Bar(
                    (
                        datetime(
                            year=bar.date.year,
                            month=bar.date.month,
                            day=bar.date.day,
                            tzinfo=timezone(timedelta(hours=-4), "EST"),
                        )
                        if not isinstance(bar.date, datetime)
                        else bar.date
                    ),
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

    def _update_data(self, bars: BarDataList, hasNewBar):
        bar = bars[-1]
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
            logger.debug(
                "remove item {} - {} - {}",
                self.bars.contract,
                self.bars.barSizeSetting,
                self.bars[0].date,
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
        Resolution.day: BarSize.Day,
        Resolution.Week: BarSize.Week,
        Resolution.Month: BarSize.Month,
    }

    def __init__(self):
        self.ib = IB()
        self.lock = asyncio.Lock()
        self.cache = TTLCache(maxsize=90, ttl=timedelta(hours=1).seconds)

    async def _reconnect(self):
        async with self.lock:
            if self.ib.client.connState == Client.DISCONNECTED:
                await self.ib.connectAsync(port=14001, readonly=True)

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
            useRTH=True,
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
                first = (
                    datetime(
                        year=bar.date.year,
                        month=bar.date.month,
                        day=bar.date.day,
                        tzinfo=timezone(timedelta(hours=-4), "EST"),
                    )
                    if not isinstance(bar.date, datetime)
                    else bar.date
                )

                if first.timestamp() <= from_:
                    return l

        start = datetime.now()
        listener = await self._subscribe(
            Stock(symbol, "SMART", "USD"),
            "" if realtime else datetime.fromtimestamp(to),
            to_duration(to - from_),
            self.mapping[freq].value,
            realtime,
        )
        logger.debug("subscribe time {}", datetime.now() - start)
        if realtime:
            self.cache[key] = listener

        return listener


def to_duration(delta: int) -> str:
    duration = timedelta(seconds=delta)
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
