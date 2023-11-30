import asyncio
from datetime import datetime, timedelta
from typing import Dict, List, Protocol, Set
from cachetools import TTLCache
from ib_insync import IB, BarData, BarDataList, Client, Contract
from eventkit import Event
import ib_insync
from loguru import logger
import pytz
from backend.broker.ib.util import get_symbol, timedelta_to_duration_str
from backend.utils.magic import SingletonABCMeta


class WatcherProtocol(Protocol):
    def id() -> str:
        ...

    def on_bar_update(bars: BarData, hasNewBar):
        ...

    def reset():
        ...


class ExtendCache(TTLCache):
    class Item:
        def __init__(self, bars: BarDataList) -> None:
            self.bars = bars
            pass

        def destroy(self):
            logger.warning(f"destroy {self.bars.contract} {self.bars.barSizeSetting}")
            SubscribeManager().ib.cancelHistoricalData(self.bars)
            SubscribeManager()._reset_watcher(
                self.bars.contract, self.bars.barSizeSetting
            )

    def popitem(self):
        k, v = super().popitem()
        logger.warning(
            f"destroy cache item {k}, {v.bars.contract} {v.bars.barSizeSetting}"
        )
        v.destroy()


class SubscribeManager(metaclass=SingletonABCMeta):
    def __init__(self, realtime: bool = True, subscribers_limit=40) -> None:
        self.watchers: Dict[str, Set[WatcherProtocol]] = dict()
        self.realtime: bool = realtime
        self.subscribers: Dict[str, ExtendCache.Item] = ExtendCache(
            maxsize=subscribers_limit, ttl=timedelta(hours=1).total_seconds()
        )
        self.ib = IB()
        self.ib.errorEvent += self._on_error
        self.conn_lock = asyncio.Lock()
        self.sub_lock = asyncio.Lock()

    async def _connect(self) -> None:
        if (
            self.ib.isConnected() == False
            and self.ib.client.connState != Client.CONNECTING
        ):
            await self.conn_lock.acquire()
            # double check
            if (
                self.ib.isConnected() == False
                and self.ib.client.connState != Client.CONNECTING
            ):
                logger.warning(f"{self} connecting to IB")
                self.subscribers.clear()
                await self.ib.connectAsync("127.0.0.1", 4001, clientId=999)
            self.conn_lock.release()

    def _on_error(self, reqId, errorCode, errorString, contract):
        if errorCode == 1102:
            # "Connectivity between IB and Trader Workstation has been
            # restored": Resubscribe to account summary.
            self.subscribers.clear()
            for k, v in self.watchers.items():
                for w in v:
                    w.reset()

    def _cache_key(self, contract: Contract, barSize: str):
        return f"{get_symbol(contract)}-{barSize}"

    def add_watcher(self, contract: Contract, barSize: str, watcher: WatcherProtocol):
        cache_key = self._cache_key(contract, barSize)
        if self.watchers.get(cache_key, None) == None:
            self.watchers[cache_key] = set()

        logger.warning(f"add_watcher {cache_key} {watcher}")
        self.watchers[cache_key].add(watcher)

    def remove_watcher(
        self, contract: Contract, barSize: str, watcher: WatcherProtocol
    ):
        cache_key = self._cache_key(contract, barSize)

        candicates = [
            w for w in self.watchers.get(cache_key, set()) if w.id() == watcher.id()
        ]
        for c in candicates:
            self.watchers[cache_key].remove(c)

    def get_watcher(self, contract: Contract, barSize: str, id: str):
        cache_key = self._cache_key(contract, barSize)

        for w in self.watchers.get(cache_key, set()):
            if w.id() == id:
                return w
        return None

    def _update_data(self, bars: BarDataList, hasNewBar):
        watchers = self.watchers.get(
            self._cache_key(bars.contract, bars.barSizeSetting), set()
        )
        for w in watchers:
            w.on_bar_update(bars[-1], hasNewBar)

    def _proceed_data(self, contract: Contract, barSize: str, bar: BarData, hasNewBar):
        cache_key = self._cache_key(contract, barSize)

        watchers = self.watchers.get(cache_key, set())
        for w in watchers:
            w.on_bar_update(bar, hasNewBar)

    def _reset_watcher(self, contract: Contract, barSize: str):
        cache_key = self._cache_key(contract, barSize)

        logger.warning(f"reset watcher {cache_key} {barSize}")
        watchers = self.watchers.get(cache_key, set())
        for w in watchers:
            w.reset()

    def raw_bars(self, contract: Contract, barSize: str):
        cache_key = self._cache_key(contract, barSize)
        return self.subscribers.get(cache_key).bars

    async def subscribe(
        self, contract: Contract, barSize: str, from_: int, to: int, countBack: int
    ) -> (List[BarData], bool):
        await self.sub_lock.acquire()
        data, bol = await self._subscribe(contract, barSize, from_, to, countBack)
        self.sub_lock.release()

        return data, bol

    async def _subscribe(
        self, contract: Contract, barSize: str, from_: int, to: int, countBack: int
    ) -> (List[BarData], bool):
        cache_key = self._cache_key(contract, barSize)

        await self._connect()

        use_cache = (
            datetime.now().timestamp() - to < timedelta(days=365).total_seconds()
        ) and self.realtime

        subscriber = None
        if use_cache:
            subscriber = self.subscribers.get(cache_key, None)

        if subscriber is not None:
            if (
                len(subscriber.bars) > 0
                and datetime.fromisoformat(
                    subscriber.bars[0].date.isoformat()
                ).timestamp()
                > from_
            ):
                logger.warning(
                    f"bar_size {barSize}: {datetime.fromisoformat(subscriber.bars[0].date.isoformat()).timestamp()} > {from_}"
                )
                subscriber.destroy()
                subscriber = None

        new_subscribe = False
        ib_bars = []
        if subscriber is None:
            ib_bars = await self.ib.reqHistoricalDataAsync(
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
            logger.warning(f"{cache_key}, use_cache: {use_cache}")
            if use_cache:
                for bar in ib_bars:
                    self._proceed_data(contract, barSize, bar, True)
                ib_bars.updateEvent += self._update_data
                self.subscribers[cache_key] = ExtendCache.Item(ib_bars)
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
        # df = ib_insync.util.df(ib_bars[:5] + ib_bars[-6:])
        # df is not None and not df.empty and logger.debug(f"ib bars:\n {df}")
        return ib_bars, new_subscribe
