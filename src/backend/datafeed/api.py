import datetime
from enum import Enum
from typing import Any, List, Protocol, LiteralString
from backend.curd.sqllite.model import SymbolExecutor
from alpaca.data.requests import StockBarsRequest
from alpaca.data.timeframe import TimeFrame
from alpaca.data import StockHistoricalDataClient

from backend.datafeed.trading_view import (
    Bar,
    LibrarySymbolInfo,
    Market,
    PeriodParams,
    SearchSymbolResultItem,
    SeriesFormat,
    SymbolType,
)

stock_client = StockHistoricalDataClient(
    "AKA93VVW12633ONJUZP9", "UkU9AuHtoTrQPrwa43iTAuEscX7JOHoxIGwGXuhc"
)


class DataFeed:
    async def search_symbols(
        user_input: LiteralString,
        exchange: LiteralString,
        type: SymbolType,
        executor: SymbolExecutor,
    ) -> List[SearchSymbolResultItem]:
        symbols = await executor.select_symbols(
            user_input=user_input, exchange=exchange, type=type
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
        exchange: LiteralString,
        type: LiteralString,
        symbol: LiteralString,
        executor: SymbolExecutor,
    ):
        symbols = await executor.resolve_symbol(
            exchange=exchange,
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
    ) -> List[Bar]:
        mapping = {"1D": TimeFrame.Day, "1M": TimeFrame.Month}
        request_params = StockBarsRequest(
            symbol_or_symbols=symbol_info.name,
            timeframe=TimeFrame.Day,
            start=datetime.date.fromtimestamp(period_params.from_),
            end=datetime.date.fromtimestamp(period_params.to),
            limit=period_params.countBack,
        )

        try:
            bars = stock_client.get_stock_bars(request_params)

            return [
                Bar(
                    time=bar.timestamp.timestamp() * 1000,
                    close=bar.close,
                    open=bar.open,
                    high=bar.high,
                    low=bar.low,
                    volume=bar.volume,
                )
                for bar in bars[symbol_info.name]
            ]
        except:
            return []
