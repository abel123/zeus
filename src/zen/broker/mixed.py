from datetime import date, datetime, timedelta, timezone
import zen_core

from broker import ib
from broker.enums import Resolution
from shelved_cache import cachedasyncmethod
from shelved_cache import PersistentCache
from cachetools import LRUCache


class Mixed:
    def __init__(self):
        self.ib = ib.Broker()

    def __del__(self):
        self.ib.ib.disconnect()

    async def subscribe(
        self,
        symbol: str,
        freq: Resolution,
        from_: int,
        to: int,
        cout_back: int | None,
        local: bool,
    ):
        [exchange, symbol] = symbol.split(":")[:2]
        if local:
            d = date.today() - timedelta(days=1)
            to = datetime(
                year=d.year,
                month=d.month,
                day=d.day,
                hour=18,
                tzinfo=timezone(timedelta(hours=-4), "EST"),
            ).timestamp()
            offset = {
                Resolution.Min: timedelta(hours=8),
                Resolution.Min3: timedelta(days=8),
                Resolution.Min5: timedelta(days=8),
                Resolution.minute_10: timedelta(days=12),
                Resolution.Min15: timedelta(days=12),
                Resolution.Min30: timedelta(days=200),
                Resolution.Hour: timedelta(days=300 / 6),
                Resolution.Day: timedelta(days=300),
                Resolution.Week: timedelta(days=300 * 7),
                Resolution.Month: timedelta(days=20 * 365),
                Resolution.year: timedelta(days=20 * 365),
            }
            return await self._local_subscribe(
                symbol=symbol,
                freq=freq,
                from_=to - int(offset[freq].total_seconds()),
                to=to,
            )
        listener = await self.ib.subscribe(
            symbol=symbol, freq=freq, from_=from_, to=to, non_realtime=False
        )
        return listener

    @cachedasyncmethod(
        cache=lambda self: PersistentCache(LRUCache, "bar.cache", maxsize=20000)
    )
    async def _local_subscribe(
        self,
        symbol: str,
        freq: Resolution,
        from_: int,
        to: int,
    ):
        listener = await self.ib.subscribe(
            symbol=symbol, freq=freq, from_=from_, to=to, non_realtime=True
        )
        listener.ib = None
        listener.zen = None
        return listener
