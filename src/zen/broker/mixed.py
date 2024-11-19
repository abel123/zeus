from datetime import date, datetime, timedelta, timezone
import pickle
from loguru import logger

from broker import futu, ib
from broker.enums import Resolution
from shelved_cache import cachedasyncmethod
from shelved_cache import PersistentCache
from cachetools import TTLCache


class Mixed:
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

    def __init__(self):
        self.ib = ib.Broker()
        self.futu = futu.Broker()

        # FIXME: adjust size accordingly
        self.pc = PersistentCache(
            TTLCache, "bar.cache", maxsize=4000, ttl=timedelta(days=1).total_seconds()
        )

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
        if "." in symbol:
            [exchange, symbol] = symbol.split(":")[:2]
            listener = await self.futu.subscribe(
                symbol=symbol,
                freq=freq,
                from_=from_,
                to=to,
                non_realtime=(cout_back == -1),
            )
            return listener

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

            return await self._local_subscribe(
                symbol=symbol,
                freq=freq,
                from_=to - int(self.offset[freq].total_seconds()),
                to=to,
            )
        listener = await self.ib.subscribe(
            symbol=symbol, freq=freq, from_=from_, to=to, non_realtime=(cout_back == -1)
        )
        return listener

    @cachedasyncmethod(cache=lambda self: self.pc)
    async def _local_subscribe_load(
        self,
        symbol: str,
        freq: Resolution,
        from_: int,
        to: int,
    ):
        logger.debug(
            "local subscribe {} {} {} {}",
            symbol,
            freq,
            datetime.fromtimestamp(
                from_,
                tz=timezone(timedelta(hours=+8), "CST"),
            ),
            datetime.fromtimestamp(
                to,
                tz=timezone(timedelta(hours=+8), "CST"),
            ),
        )
        listener = await self.ib.subscribe(
            symbol=symbol, freq=freq, from_=from_, to=to, non_realtime=True
        )
        listener.ib = None
        listener.zen = None
        return pickle.dumps(listener, protocol=pickle.HIGHEST_PROTOCOL)

    async def _local_subscribe(
        self,
        symbol: str,
        freq: Resolution,
        from_: int,
        to: int,
    ):
        load = await self._local_subscribe_load(
            symbol=symbol, freq=freq, from_=from_, to=to
        )
        return pickle.loads(load)
