import asyncio
from datetime import datetime, timedelta
from typing import Dict, List, Protocol, Set
from cachetools import TTLCache
from ib_insync import IB, BarData, BarDataList, Client, Contract
from eventkit import Event
import ib_insync
from loguru import logger
import pytz
from backend.broker.ib.util import timedelta_to_duration_str
from backend.calculate.protocol import Symbol, SymbolType, WatcherProtocol
from backend.utils.model_convert import to_czsc_bar
from backend.utils.magic import SingletonABCMeta
from backend.utils.options import Options
from czsc.enum import Freq


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
    freq_map = {
        "1 day": Freq.D,
        "1 month": Freq.M,
        "1 week": Freq.W,
        "4 hours": Freq.F480,
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

    czsc_to_ib_map = {
        Freq.D: "1 day",
        Freq.M: "1 month",
        Freq.W: "1 week",
        Freq.F240: "4 hours",
        Freq.F120: "2 hours",
        Freq.F60: "1 hour",  # 需要特殊处理, trading view sdk内部会进行合并
        Freq.F30: "30 mins",
        Freq.F20: "20 mins",
        Freq.F15: "15 mins",
        Freq.F10: "10 mins",
        Freq.F5: "5 mins",
        Freq.F3: "3 mins",
        Freq.F2: "2 mins",
        Freq.F1: "1 min",
    }

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
            try:
                # double check
                if (
                    self.ib.isConnected() == False
                    and self.ib.client.connState != Client.CONNECTING
                ):
                    logger.warning(f"{self} connecting to IB")
                    self.subscribers.clear()
                    await self.ib.connectAsync("127.0.0.1", 4001, clientId=999)
                self.conn_lock.release()
            except Exception as e:
                logger.exception(f"connect except {e}")
                self.conn_lock.release()

    def _on_error(self, reqId, errorCode, errorString, contract):
        logger.error(
            f"ib error reqId: {reqId}, errCode:{errorCode}, errString: {errorString}, contract: {contract}"
        )
        if errorCode == 1102:
            # "Connectivity between IB and Trader Workstation has been
            # restored": Resubscribe to account summary.
            self.subscribers.clear()
            for k, v in self.watchers.items():
                for w in v:
                    w.reset()

    def _cache_key(self, contract: str, barSize: str):
        if isinstance(contract, Contract):
            return f"{self._get_symbol(contract)}-{barSize}"

        return f"{contract}-{barSize}"

    def upsert_watcher(self, watcher: WatcherProtocol):
        for c in watcher.contracts():
            barSize = self.czsc_to_ib_map.get(c[1])
            cache_key = self._cache_key(c[0].raw, barSize)
            if self.watchers.get(cache_key, None) == None:
                self.watchers[cache_key] = set()

            logger.warning(f"upsert_watcher {cache_key} {watcher.id()}-{watcher}")
            self.watchers[cache_key].add(watcher)

    def remove_watcher(self, watcher: WatcherProtocol):
        for c in watcher.contracts():
            barSize = self.czsc_to_ib_map.get(c[1])
            cache_key = self._cache_key(c[0].raw, barSize)

            candicates = [
                w for w in self.watchers.get(cache_key, set()) if w.id() == watcher.id()
            ]
            for c in candicates:
                self.watchers[cache_key].remove(c)

    def get_watcher(self, symbol: Symbol, freq: Freq, id: str):
        barSize = self.czsc_to_ib_map.get(freq)
        cache_key = self._cache_key(symbol.raw, barSize)

        for w in self.watchers.get(cache_key, set()):
            if w.id() == id:
                return w
        return None

    def _get_symbol(self, contract: Contract) -> str:
        return contract.symbol if contract.symbol != "" else contract.localSymbol

    def _update_data(self, bars: BarDataList, hasNewBar):
        watchers = self.watchers.get(
            self._cache_key(bars.contract, bars.barSizeSetting), set()
        )
        bar = to_czsc_bar(
            Symbol(raw=self._get_symbol(bars.contract), type=SymbolType.STOCK),
            self.freq_map.get(bars.barSizeSetting),
            bars[-1],
        )

        for w in watchers:
            w.on_bar_update(bar, hasNewBar)

    def _proceed_data(self, contract: Contract, barSize: str, bar: BarData, hasNewBar):
        cache_key = self._cache_key(contract, barSize)

        watchers = self.watchers.get(cache_key, set())

        bar = to_czsc_bar(
            Symbol(raw=self._get_symbol(contract), type=SymbolType.STOCK),
            self.freq_map.get(barSize),
            bar,
        )

        for w in watchers:
            w.on_bar_update(bar, hasNewBar)

    def _reset_watcher(self, contract: Contract, barSize: str):
        cache_key = self._cache_key(self._get_symbol(contract), barSize)

        logger.warning(f"reset watcher {cache_key} {barSize}")
        watchers = self.watchers.get(cache_key, set())
        for w in watchers:
            w.reset()

    def raw_bars(self, symbol: Symbol, freq: Freq):
        barSize = self.czsc_to_ib_map.get(freq)
        cache_key = self._cache_key(symbol.raw, barSize)
        return self.subscribers.get(cache_key).bars

    async def subscribe(
        self, symbol: Symbol, freq: Freq, from_: int, to: int, countBack: int
    ) -> (List[BarData], bool):
        # await self.sub_lock.acquire()

        data, bol = await self._subscribe(symbol, freq, from_, to, countBack)
        # self.sub_lock.release()

        return data, bol

    async def _subscribe(
        self, symbol: Symbol, freq: Freq, from_: int, to: int, countBack: int
    ) -> (List[BarData], bool):
        barSize = self.czsc_to_ib_map.get(freq)
        contract = ib_insync.Stock(
            symbol.raw,
            "SMART",
            "USD",
            primaryExchange="NASDAQ" if symbol.exchange == "" else symbol.exchange,
        )

        if symbol.type == SymbolType.OPTION:
            contract = ib_insync.Option(
                exchange="SMART",
                localSymbol=symbol.raw,
                primaryExchange="CBOE",
            )
        cache_key = self._cache_key(symbol.raw, barSize)

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
                useRTH=(
                    contract.symbol != "TSLA"
                    or Options().config().enable_overnight == False
                ),
                formatDate=1,
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
