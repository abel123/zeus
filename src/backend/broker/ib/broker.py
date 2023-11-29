import calendar
from collections import OrderedDict
from datetime import date, datetime, timedelta, timezone
import math
from typing import Any, List
from cachetools import LRUCache, TLRUCache, TTLCache
from ib_insync import IB
import ib_insync
from loguru import logger
import pytz
from backend.calculate.zen import signal
from backend.calculate.zen.signal.macd import MACDArea

from backend.datafeed.tv_model import Bar, LibrarySymbolInfo, MacdConfig, PeriodParams
from czsc.analyze import CZSC
from czsc.enum import Freq
from czsc.objects import RawBar
from asyncache import cached
from cachetools import TTLCache


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
    freq_map = {
        "1 day": Freq.D,
        "1 month": Freq.M,
        "1 week": Freq.W,
        "2 hours": Freq.F120,
        "1 hour": Freq.F60,
        "30 mins": Freq.F30,
        "20 mins": Freq.F20,
        "15 mins": Freq.F15,
        "10 mins": Freq.F10,
        "5 mins": Freq.F5,
        "3 mins": Freq.F3,
        "2 mins": Freq.F2,
        "1 min": Freq.F1,
    }

    class CacheItem:
        def __init__(
            self,
            bars: ib_insync.BarDataList,
            macd_config=signal.macd.Config(
                macd_config=[MacdConfig(fast=12, slow=26, signal=9)]
            ),
        ) -> None:
            self.macd_signal = MACDArea(macd_config)
            raw_bars = [
                RawBar(
                    symbol=bars.contract.symbol,
                    id=datetime.fromisoformat(bar.date.isoformat()).timestamp(),
                    dt=datetime.fromisoformat(bar.date.isoformat()),
                    freq=Broker.freq_map.get(bars.barSizeSetting, Freq.D),
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
            self.czsc = CZSC(
                raw_bars,
                get_signals=self.macd_signal.macd_area_bc,
                on_bi_break=self.macd_signal.on_bi_break,
                on_bi_create=self.macd_signal.on_bi_create,
            )
            self.bars.updateEvent += self.on_bar_update

        def on_bar_update(self, bars: ib_insync.BarDataList, hasNewBar):
            # logger.debug(f"update {bars.contract.symbol} {bars[-1]}")
            bar = bars[-1]
            self.czsc.update(
                RawBar(
                    symbol=bars.contract.symbol,
                    id=datetime.fromisoformat(bar.date.isoformat()).timestamp(),
                    dt=datetime.fromisoformat(bar.date.isoformat()),
                    freq=Broker.freq_map.get(bars.barSizeSetting, Freq.D),
                    open=bar.open,
                    close=bar.close,
                    high=bar.high,
                    low=bar.low,
                    vol=bar.volume,
                    amount=0.0,
                )
            )

        def destroy(self):
            logger.warning(f"destroy {self.bars.barSizeSetting} - {self.bars.contract}")
            Broker.ib.cancelHistoricalData(self.bars)

    class RequesterCache(TTLCache):
        def popitem(self):
            k, v = super().popitem()
            v.destroy()

    ib: IB = IB()
    cache = RequesterCache(maxsize=40, ttl=timedelta(hours=6).total_seconds())
    last_macd_config = OrderedDict()

    async def init():
        if (
            Broker.ib.isConnected() == False
            and Broker.ib.client.connState != Broker.ib.client.CONNECTING
        ):
            logger.warning("reconnecting to IB")
            Broker.cache.clear()
            await Broker.ib.connectAsync(
                "127.0.0.1", 4001, clientId=999  # Broker.ib.client.clientId + 2
            )

    @cached(LRUCache(1024))
    async def get_head_time(symbol_info: LibrarySymbolInfo) -> int:
        contract = ib_insync.Stock(
            symbol_info.name,
            "SMART",
            "USD",
            primaryExchange="NASDAQ"
            if symbol_info.exchange == ""
            else symbol_info.exchange,
        )
        ts = await Broker.ib.reqHeadTimeStampAsync(contract, "TRADES", True, 2)
        return ts

    async def get_bars(
        symbol_info: LibrarySymbolInfo,
        resolution: str,
        period_params: PeriodParams,
        macd_config: List[MacdConfig],
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
            "1W": "1 week",
            "240": "4 hours",
            "180": "2 hours",
            "120": "2 hours",
            "60": "1 hour",
            "30": "30 mins",
            "20": "20 mins",
            "15": "15 mins",
            "10": "10 mins",
            "5": "5 mins",
            "3": "3 mins",
            "2": "2 mins",
            "1": "1 min",
        }

        try:
            await Broker.init()

            contract = ib_insync.Stock(
                symbol_info.name,
                "SMART",
                "USD",
                primaryExchange="NASDAQ"
                if symbol_info.exchange == ""
                else symbol_info.exchange,
            )

            if symbol_info.exchange == "option":
                contract = ib_insync.Option(
                    exchange="SMART",
                    localSymbol=symbol_info.name,
                    primaryExchange="CBOE",
                )
            # [contract] = await Broker.ib.qualifyContractsAsync(contract)
            cache_key = f"{resolution} - {macd_config} {str(contract)}"

            logger.debug(
                {
                    "contract ": contract,
                    "key": cache_key,
                    "connect_state": Broker.ib.client.connState,
                }
            )
            use_cache = (
                datetime.now().timestamp() - period_params.to
                < timedelta(days=365).total_seconds()
            )
            if len(macd_config) == 0:
                if resolution in Broker.last_macd_config:
                    macd_config = Broker.last_macd_config[resolution]
            else:
                logger.debug(
                    f"++++++++++++{str(macd_config)} {str(Broker.last_macd_config.get(resolution))}"
                )
                if resolution in Broker.last_macd_config and set(macd_config) <= set(
                    Broker.last_macd_config[resolution]
                ):
                    use_cache = True
                else:
                    use_cache = False
                if not use_cache:
                    Broker.last_macd_config[resolution] = macd_config

            ib_bars = []
            requester: Broker.CacheItem = None
            if use_cache:
                requester = Broker.cache.get(cache_key)

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

            logger.debug(
                {
                    "use_cache": use_cache,
                    "peroid": period_params,
                    "durationStr": timedelta_to_duration_str(
                        (
                            datetime.now(pytz.utc)
                            if use_cache
                            else datetime.fromtimestamp(period_params.to, pytz.utc)
                        )
                        - datetime.fromtimestamp(period_params.from_, pytz.utc)
                    ),
                    "config": macd_config,
                }
            )
            if requester is None:
                ib_bars = await Broker.ib.reqHistoricalDataAsync(
                    contract,
                    endDateTime=""
                    if use_cache
                    else datetime.fromtimestamp(period_params.to),
                    durationStr=timedelta_to_duration_str(
                        (
                            datetime.now(pytz.utc) + timedelta(days=2)
                            if use_cache
                            else datetime.fromtimestamp(period_params.to, pytz.utc)
                        )
                        - datetime.fromtimestamp(period_params.from_, pytz.utc)
                    ),
                    barSizeSetting=mapping.get(resolution),
                    whatToShow="TRADES",
                    useRTH=True,
                    formatDate=2,
                    keepUpToDate=use_cache,
                )

                requester = Broker.CacheItem(
                    ib_bars,
                    signal.Config(macd_config=macd_config),
                )
                if use_cache:
                    Broker.cache.update([(cache_key, requester)])
            else:
                requester.macd_signal.add_key(config=macd_config)
                logger.debug(
                    {
                        "caching": True,
                        "start": requester.bars[0].date,
                        "end": requester.bars[-1].date,
                    }
                )
                if period_params.countBack is None:
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
                else:
                    if not isinstance(requester.bars[0].date, datetime):
                        ib_bars = [
                            bar
                            for bar in requester.bars
                            if datetime.fromisoformat(bar.date.isoformat())
                            <= datetime.fromtimestamp(period_params.to)
                        ]
                    else:
                        ib_bars = [
                            bar
                            for bar in requester.bars
                            if datetime.fromisoformat(bar.date.isoformat())
                            <= datetime.fromtimestamp(period_params.to, pytz.utc)
                        ]
                    ib_bars = ib_bars[max(len(ib_bars) - period_params.countBack, 0) :]
            # df = ib_insync.util.df(requester.bars[:5] + requester.bars[-6:])
            # df is not None and not df.empty and logger.debug(f"ib bars:\n {df}")
            return [
                Bar(
                    time=bar.date.timestamp()
                    if isinstance(bar.date, datetime)
                    else datetime.utcfromtimestamp(
                        calendar.timegm(bar.date.timetuple())
                    ).timestamp()
                    + timedelta(hours=1).total_seconds(),
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
            # logger.warning(f"exception =========={e}")
            raise e
            return None, None

        ...
