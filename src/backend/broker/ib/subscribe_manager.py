from datetime import datetime, timedelta
from typing import Dict, List, Protocol, Set
from cachetools import TTLCache
from ib_insync import IB, BarData, BarDataList, Client, Contract
from eventkit import Event
from loguru import logger
import pytz
from backend.broker.ib.util import timedelta_to_duration_str
from backend.utils.magic import SingletonABCMeta


class WatcherProtocol(Protocol):
    def on_bar_update(bars: BarDataList, hasNewBar):
        ...

    def reset():
        ...


class ExtendCache(TTLCache):
    class Item:
        def __init__(self, bars: BarDataList) -> None:
            self.bars = bars
            pass

        def destroy(self):
            SubscribeManager().ib.cancelHistoricalData(self.bars)
            SubscribeManager()._reset_watcher(self.bars.barSizeSetting)

    def popitem(self):
        k, v = super().popitem()
        v.destroy()


class SubscribeManager(object):
    __metaclass__ = SingletonABCMeta

    def __init__(self, realtime: bool = True, subscribers_limit=40) -> None:
        self.watchers: Dict[str, Set[WatcherProtocol]] = dict()
        self.realtime: bool = realtime
        self.subscribers: Dict[str, ExtendCache.Item] = ExtendCache(
            maxsize=subscribers_limit, ttl=timedelta(hours=1).total_seconds()
        )
        self.ib = IB()

    async def init(self):
        if (
            self.ib.isConnected() == False
            and self.ib.client.connState != Client.CONNECTING
        ):
            logger.warning("connecting to IB")
            self.subscribers.clear()
            await self.ib.connectAsync("127.0.0.1", 4001, clientId=999)

    def add_watcher(self, barSize: str, watcher: WatcherProtocol):
        if self.watchers.get(barSize, None) == None:
            self.watchers[barSize] = set()
        self.watchers[barSize].add(watcher)

    def remove_watcher(self, barSize: str, watcher: WatcherProtocol):
        self.watchers[barSize].remove(watcher)

    def _update_data(self, bars: BarDataList, hasNewBar):
        watchers = self.watchers.get(bars.barSizeSetting, set())
        for w in watchers:
            w.on_bar_update(bars, hasNewBar)

    def _reset_watcher(self, barSize: str):
        watchers = self.watchers.get(barSize, set())
        for w in watchers:
            w.reset()

    async def subscribe(
        self, contract: Contract, barSize: str, from_: int, to: int, countBack: int
    ) -> (List[BarData], bool):
        use_cache = (
            datetime.now().timestamp() - to < timedelta(days=365).total_seconds()
        ) and self.realtime

        subscriber = None
        if use_cache:
            subscriber = self.subscribers.get(barSize, None)

        if subscriber is not None:
            if (
                len(subscriber.bars) > 0
                and datetime.fromisoformat(
                    subscriber.bars[0].date.isoformat()
                ).timestamp()
                > from_
            ):
                subscriber.destroy()
                subscriber = None

        new_subscribe = False
        if subscriber is None:
            bars = await self.ib.reqHistoricalDataAsync(
                contract,
                endDateTime="" if use_cache else datetime.fromtimestamp(to),
                durationStr=timedelta_to_duration_str(
                    (
                        datetime.now(pytz.utc) + timedelta(days=2)
                        if use_cache
                        else datetime.fromtimestamp(to, pytz.utc)
                    )
                    - datetime.fromtimestamp(from_, pytz.utc)
                ),
                barSizeSetting=barSize,
                whatToShow="TRADES",
                useRTH=True,
                formatDate=2,
                keepUpToDate=use_cache,
            )

            if use_cache:
                bars.updateEvent += self._update_data
                self.subscribers[barSize] = ExtendCache.Item(bars)
                new_subscribe = True
        else:
            if countBack is None:
                if not isinstance(subscriber.bars[0].date, datetime):
                    ib_bars = [
                        bar
                        for bar in subscriber.bars
                        if datetime.fromisoformat(bar.date.isoformat())
                        >= datetime.fromtimestamp(from_)
                        and datetime.fromisoformat(bar.date.isoformat())
                        <= datetime.fromtimestamp(to)
                    ]
                else:
                    ib_bars = [
                        bar
                        for bar in subscriber.bars
                        if datetime.fromisoformat(bar.date.isoformat())
                        >= datetime.fromtimestamp(from_, pytz.utc)
                        and datetime.fromisoformat(bar.date.isoformat())
                        <= datetime.fromtimestamp(to, pytz.utc)
                    ]
            else:
                if not isinstance(subscriber.bars[0].date, datetime):
                    ib_bars = [
                        bar
                        for bar in subscriber.bars
                        if datetime.fromisoformat(bar.date.isoformat())
                        <= datetime.fromtimestamp(to)
                    ]
                else:
                    ib_bars = [
                        bar
                        for bar in subscriber.bars
                        if datetime.fromisoformat(bar.date.isoformat())
                        <= datetime.fromtimestamp(to, pytz.utc)
                    ]
                ib_bars = ib_bars[max(len(ib_bars) - countBack, 0) :]
        return ib_bars, new_subscribe

    ...
