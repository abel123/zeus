import datetime
from enum import Enum
import math
from typing import Any, List, Protocol, LiteralString

from ib_insync import IB
from loguru import logger
from backend.broker.ib.broker import Broker
from backend.broker.futu.broker import Broker as FutuBroker

from backend.broker.ib.options import get_tsla_option_list
from backend.broker.ib.signals import Watcher
from backend.curd.sqllite.model import SymbolExecutor

from backend.datafeed.tv_model import (
    Bar,
    LibrarySymbolInfo,
    MacdConfig,
    Market,
    PeriodParams,
    SearchSymbolResultItem,
    SeriesFormat,
    SymbolType,
)


class DataFeed:
    async def init():
        ...
    
    async def search_symbols(
        user_input: LiteralString,
        screener: LiteralString,
        type: SymbolType,
        executor: SymbolExecutor,
    ) -> List[SearchSymbolResultItem]:
        symbols = await executor.select_symbols(
            user_input=user_input, screener=screener, type=type
        )
        extras: List[SearchSymbolResultItem] = []
        if user_input == "%TSLA%" or "%TSLA " in user_input:
            contracts = await get_tsla_option_list()
            logger.debug(f"contracts {contracts}")
            extras = [
                SearchSymbolResultItem(
                    symbol=c.localSymbol,
                    full_name=c.localSymbol,
                    description=f"tsla {c.lastTradeDateOrContractMonth} - {"PUT" if c.right == "P" else "CALL"} {c.strike} option",
                    exchange=c.exchange,
                    ticker="option:" + c.localSymbol,
                    type="option",
                )
                for c in contracts
            ]
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
        ] + (extras)

    async def resolve_symbol(
        screener: LiteralString,
        type: LiteralString,
        symbol: LiteralString,
        executor: SymbolExecutor,
    ):
        if type == "option":
            return [
                LibrarySymbolInfo(
                    name=symbol,
                    ticker=f"option:{symbol}",
                    full_name=f"option:{symbol}",
                    description="",
                    exchange="NASDAQ",
                    type="option",
                    session=Market["america"]["session"],
                    timezone=Market["america"]["timezone"],
                    listed_exchange="NASDAQ",
                    format=SeriesFormat.price,
                    pricescale=100,
                    minmov=1,  ###
                    minmove2=1,  ###
                    supported_resolutions=[
                        "1",
                        "5",
                        "15",
                        "60",
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
                    has_weekly_and_monthly=False,
                )
            ]

        symbols = await executor.resolve_symbol(
            screener=screener,
            type=type,
            symbol=symbol,
        )

        return [
            LibrarySymbolInfo(
                name=sym.symbol,
                ticker=f"{sym.exchange}:{sym.symbol}" if screener == "america" else sym.symbol,
                full_name=f"{sym.exchange}:{sym.symbol}" if screener == "america" else sym.symbol,
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
                    "3",
                    "5",
                    "10",
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
        symbol_info: LibrarySymbolInfo,
        resolution: str,
        period_params: PeriodParams,
        macd_config: List[MacdConfig] = [],
    ) -> (List[Bar], Watcher):
        if symbol_info.exchange in ["SSE", "HKEX", "SZSE"]:
             return await FutuBroker.get_bars(
                symbol_info=symbol_info,
                resolution=resolution,
                period_params=period_params,
                macd_config=macd_config,
            )
        return await Broker.get_bars(
            symbol_info=symbol_info,
            resolution=resolution,
            period_params=period_params,
            macd_config=macd_config,
        )
