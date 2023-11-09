from datetime import date, datetime, timedelta, timezone
import math
from typing import Any, List
from cachetools import TLRUCache, TTLCache
from ib_insync import IB
import ib_insync
import pytz
from backend.calculate.zen import signal
from backend.calculate.zen.signal.macd import MACDArea

from backend.datafeed.tv_model import Bar, LibrarySymbolInfo, PeriodParams
from czsc.analyze import CZSC
from czsc.enum import Freq
from czsc.objects import RawBar


def timedelta_to_duration_str(duration: timedelta) -> str:
    if duration.days >= 360:
        return f"{math.ceil(duration.total_seconds()*1.0 / timedelta(365).total_seconds()):.0f} Y"
    elif duration.days >= 30:
        return f"{math.ceil(duration.total_seconds()*1.0 / timedelta(30).total_seconds()):.0f} M"
    elif duration.days >= 7:
        return f"{math.ceil(duration.total_seconds()*1.0 / timedelta(7).total_seconds()):.0f} W"
    elif duration.days >= 1:
        return f"{math.ceil(duration.total_seconds()*1.0/timedelta(1).total_seconds()):.0f} D"
    else:
        return f"{max(30, duration.total_seconds()):.0f} S"


class Broker:
    class CacheItem:
        def __init__(
            self, bars: ib_insync.BarDataList, macd_config=signal.macd.Config()
        ) -> None:
            self.macd_signal = MACDArea(macd_config)
            raw_bars = [
                RawBar(
                    symbol=bars.contract.symbol,
                    id=datetime.fromisoformat(bar.date.isoformat()).timestamp(),
                    dt=datetime.fromisoformat(bar.date.isoformat()),
                    freq=Freq.D,
                    open=bar.open,
                    close=bar.close,
                    high=bar.high,
                    low=bar.low,
                    vol=bar.volume,
                    amount=0.0,
                )
                for idx, bar in enumerate(bars)
            ]
            self.bars = bars
            self.czsc = CZSC(raw_bars, get_signals=self.macd_signal.macd_area_bc)
            self.bars.updateEvent += self.on_bar_update

        def on_bar_update(self, bars: ib_insync.BarDataList, hasNewBar):
            print("update ", bars.contract.symbol, bars[-1])
            bar = bars[-1]
            self.czsc.update(
                RawBar(
                    symbol=bars.contract.symbol,
                    id=datetime.fromisoformat(bar.date.isoformat()).timestamp(),
                    dt=datetime.fromisoformat(bar.date.isoformat()),
                    freq=Freq.D,
                    open=bar.open,
                    close=bar.close,
                    high=bar.high,
                    low=bar.low,
                    vol=bar.volume,
                    amount=0.0,
                )
            )

        def destroy(self):
            print(f"destroy {self.bars.contract}")
            Broker.ib.cancelHistoricalData(self.bars)

    class RequesterCache(TTLCache):
        def popitem():
            k, v = super().popitem()
            v.destrop()

    ib = IB()
    cache = RequesterCache(maxsize=20, ttl=timedelta(minutes=12).seconds)

    async def init():
        await Broker.ib.connectAsync("127.0.0.1", 4001, clientId=991)

    async def get_bars(
        symbol_info: LibrarySymbolInfo, resolution: str, period_params: PeriodParams
    ) -> (List[Bar], CacheItem):
        """
         barSizeSetting: Time period of one bar. Must be one of:
                '1 secs', '5 secs', '10 secs' 15 secs', '30 secs',
                '1 min', '2 mins', '3 mins', '5 mins', '10 mins', '15 mins',
                '20 mins', '30 mins',
                '1 hour', '2 hours', '3 hours', '4 hours', '8 hours',
                '1 day', '1 week', '1 month'.
        }
        """
        mapping = {
            "1D": "1 day",
            "1M": "1 month",
            # "2D": TimeFrame(amount=2, unit=TimeFrameUnit.Day),
            "1W": "1 week",
            # "12M": TimeFrame(amount=12, unit=TimeFrame.Month),
            "240": "4 hours",
            "120": "2 hours",
            "60": "1 hour",
            "30": "30 mins",
            "15": "15 mins",
            "10": "10 mins",
            "5": "5 mins",
            "3": "3 mins",
            "2": "2 mins",
            "1": "1 min",
        }

        try:
            contract = ib_insync.Stock(
                symbol_info.name,
                "SMART",
                "USD",
                primaryExchange="NASDAQ"
                if symbol_info.exchange == ""
                else symbol_info.exchange,
            )
            cache_key = f"{resolution} - {str(contract)}"
            print("contract ", contract, cache_key)

            requester: Broker.CacheItem = Broker.cache.get(cache_key)
            ib_bars = []
            print(
                {
                    "from": datetime.fromtimestamp(period_params.from_, pytz.utc),
                    "to": datetime.fromtimestamp(period_params.to, pytz.utc),
                }
            )
            if requester is not None:
                if (
                    len(requester.bars) > 0
                    and datetime.fromisoformat(
                        requester.bars[0].date.isoformat()
                    ).timestamp()
                    > period_params.from_
                ):
                    requester.destroy()
                    requester = None

            if requester is None:
                print(
                    {
                        "startDateTime": datetime.fromtimestamp(
                            period_params.from_, pytz.utc
                        ),
                        "endDateTime": datetime.fromtimestamp(
                            period_params.to, pytz.utc
                        ),
                        "durationStr": timedelta_to_duration_str(
                            datetime.now(pytz.utc)
                            - datetime.fromtimestamp(period_params.from_, pytz.utc)
                        ),
                    }
                )
                ib_bars = await Broker.ib.reqHistoricalDataAsync(
                    contract,
                    endDateTime="",  # datetime.fromtimestamp(period_params.to),
                    durationStr=timedelta_to_duration_str(
                        datetime.now(pytz.utc)
                        - datetime.fromtimestamp(period_params.from_, pytz.utc)
                    ),
                    barSizeSetting=mapping.get(resolution),
                    whatToShow="TRADES",
                    useRTH=True,
                    formatDate=2,
                    keepUpToDate=True,
                )
                if resolution != "1":
                    requester = Broker.CacheItem(ib_bars)
                else:
                    requester = Broker.CacheItem(
                        ib_bars,
                        signal.Config(fastperiod=4, slowperiod=9, signalperiod=9),
                    )
                Broker.cache.update([(cache_key, requester)])
            else:
                print(" ----------------------- use cache ---------------")
                print(
                    {
                        "start": requester.bars[0].date,
                        "end": requester.bars[-1].date,
                        "from": datetime.fromtimestamp(period_params.from_, pytz.utc),
                        "to": datetime.fromtimestamp(period_params.to, pytz.utc),
                    }
                )
                if not isinstance(requester.bars[0].date, datetime):
                    ib_bars = [
                        bar
                        for bar in requester.bars
                        if datetime.fromisoformat(bar.date.isoformat())
                        >= datetime.fromtimestamp(period_params.from_)
                        and datetime.fromisoformat(bar.date.isoformat())
                        <= datetime.fromtimestamp(period_params.to)
                    ]
                else:
                    ib_bars = [
                        bar
                        for bar in requester.bars
                        if datetime.fromisoformat(bar.date.isoformat())
                        >= datetime.fromtimestamp(period_params.from_, pytz.utc)
                        and datetime.fromisoformat(bar.date.isoformat())
                        <= datetime.fromtimestamp(period_params.to, pytz.utc)
                    ]
            df = ib_insync.util.df(ib_bars[:5] + ib_bars[-5:])
            df is not None and not df.empty and print("ib bars", df)
            return [
                Bar(
                    time=datetime.fromisoformat(bar.date.isoformat()).timestamp(),
                    # bar.date.timestamp() * 1000,
                    close=bar.close,
                    open=bar.open,
                    high=bar.high,
                    low=bar.low,
                    volume=bar.volume,
                )
                for bar in ib_bars
            ], requester
        except Exception as e:
            print("exception =========={}", e)
            raise e
            return None

        ...
