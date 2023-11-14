from datetime import datetime, timedelta
from typing import List
from asyncache import cached
from ib_insync import *
from loguru import logger
from shelved_cache import PersistentCache
from cachetools import TTLCache

from backend.broker.ib.broker import Broker

filename = "./data/cache/option_list"

# create persistency around an LRUCache
pc = PersistentCache(
    TTLCache, filename=filename, maxsize=10, ttl=timedelta(days=2).total_seconds()
)


@cached(pc)
async def get_tsla_option_list() -> List[Contract]:
    try:
        ib = Broker.ib

        tsla = Stock("TSLA", "SMART", "USD")
        await ib.qualifyContractsAsync(tsla)

        his = await ib.reqHistoricalDataAsync(
            tsla, datetime.now(), "1 D", "1 hour", "TRADES", True, 2
        )
        spxValue = his[-1].close

        chains = await ib.reqSecDefOptParamsAsync(
            tsla.symbol, "", tsla.secType, tsla.conId
        )
        util.df(chains)

        chain = next(
            c for c in chains if c.tradingClass == "TSLA" and c.exchange == "SMART"
        )

        strikes = [
            strike
            for strike in chain.strikes
            if strike % 5 == 0 and spxValue - 10 < strike < spxValue + 10
        ]
        expirations = sorted(exp for exp in chain.expirations)[:2]
        rights = ["P", "C"]

        contracts = [
            Option("TSLA", expiration, strike, right, "SMART", tradingClass="TSLA")
            for expiration in expirations
            for strike in strikes
            for right in rights
        ]

        contracts = await ib.qualifyContractsAsync(*contracts)
        return contracts
    except Exception as e:
        logger.error(f"fetch option list error {e}")
        return []
