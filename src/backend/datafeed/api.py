import datetime
from enum import Enum
import math
from typing import Any, List, Protocol, LiteralString

from ib_insync import IB
import ib_insync
from backend.broker.ib.broker import Broker
from backend.curd.sqllite.model import SymbolExecutor
from alpaca.data.requests import StockBarsRequest
from alpaca.data.timeframe import TimeFrame, TimeFrameUnit

from backend.datafeed.trading_view import (
    Bar,
    LibrarySymbolInfo,
    Market,
    PeriodParams,
    SearchSymbolResultItem,
    SeriesFormat,
    SymbolType,
)


class DataFeed:
    async def init():
        await Broker.init()

    async def search_symbols(
        user_input: LiteralString,
        screener: LiteralString,
        type: SymbolType,
        executor: SymbolExecutor,
    ) -> List[SearchSymbolResultItem]:
        symbols = await executor.select_symbols(
            user_input=user_input, screener=screener, type=type
        )
        return [
            SearchSymbolResultItem(
                symbol=sym.symbol,
                full_name=sym.symbol,
                description=sym.desc,
                exchange=sym.symbol,
                ticker=sym.symbol,
                type=sym.type,
            )
            for sym in symbols
        ]

    async def resolve_symbol(
        screener: LiteralString,
        type: LiteralString,
        symbol: LiteralString,
        executor: SymbolExecutor,
    ):
        symbols = await executor.resolve_symbol(
            screener=screener,
            type=type,
            symbol=symbol,
        )

        return [
            LibrarySymbolInfo(
                name=sym.symbol,
                ticker=f"{sym.exchange}:{sym.symbol}",
                full_name=f"{sym.exchange}:{sym.symbol}",
                description=sym.desc,
                exchange=sym.exchange,
                type=sym.type,
                session=Market[sym.screener]["session"],
                timezone=Market[sym.screener]["timezone"],
                listed_exchange=sym.exchange,
                format=SeriesFormat.price,
                pricescale=sym.pricescale,
                minmov=1,  ###
                minmove2=1,  ###
                supported_resolutions=[
                    "1",
                    "5",
                    "15",
                    "30",
                    "60",
                    "1D",
                    "1W",
                    "1M",
                    "12M",
                ],
                intraday_multipliers=[
                    "1",
                    "2",
                    "3",
                    "5",
                    "10",
                    "15",
                    "20",
                    "30",
                    "60",
                    "120",
                    "240",
                ],
                seconds_multipliers=[
                    "1",
                    "2",
                    "3",
                    "5",
                    "10",
                    "15",
                    "20",
                    "30",
                    "40",
                    "50",
                    "60",
                ],
                has_intraday=True,
                has_seconds=False,
                has_daily=True,
                has_weekly_and_monthly=True,
            )
            for sym in symbols
        ]

    async def get_bars(
        symbol_info: LibrarySymbolInfo, resolution: str, period_params: PeriodParams
    ) -> (List[Bar], Broker.CacheItem):
        return await Broker.get_bars(
            symbol_info=symbol_info, resolution=resolution, period_params=period_params
        )
