import datetime
from enum import Enum
from typing import Any, List, Protocol, LiteralString
from backend.curd.sqllite.model import SymbolExecutor
from alpaca.data.requests import StockBarsRequest
from alpaca.data.timeframe import TimeFrame, TimeFrameUnit
from alpaca.data import StockHistoricalDataClient, Adjustment

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
    ) -> List[Bar]:
        """
             frequency_maps = {
            '10s': '10S', '30s': '30S',
            '1m': '1', '2m': '2', '3m': '3', '5m': '5', '10m': '10', '15m': '15',
            '30m': '30', '60m': '60', '120m': '120',
            '4h': '240',
            'd': '1D', '2d': '2D',
            'w': '1W', 'm': '1M', 'y': '12M'
        }
        """
        mapping = {
            "1D": TimeFrame.Day,
            "1M": TimeFrame.Month,
            # "2D": TimeFrame(amount=2, unit=TimeFrameUnit.Day),
            "1W": TimeFrame.Week,
            "12M": TimeFrame(amount=12, unit=TimeFrame.Month),
            "240": TimeFrame(amount=4, unit=TimeFrameUnit.Hour),
            "120": TimeFrame(amount=2, unit=TimeFrameUnit.Hour),
            "60": TimeFrame.Hour,
            "30": TimeFrame(amount=30, unit=TimeFrameUnit.Minute),
            "15": TimeFrame(amount=15, unit=TimeFrameUnit.Minute),
            "10": TimeFrame(amount=10, unit=TimeFrameUnit.Minute),
            "5": TimeFrame(amount=5, unit=TimeFrameUnit.Minute),
            "3": TimeFrame(amount=3, unit=TimeFrameUnit.Minute),
            "2": TimeFrame(amount=2, unit=TimeFrameUnit.Minute),
            "1": TimeFrame(amount=1, unit=TimeFrameUnit.Minute),
        }
        request_params = StockBarsRequest(
            symbol_or_symbols=symbol_info.name,
            timeframe=mapping[resolution],
            start=datetime.date.fromtimestamp(period_params.from_),
            end=datetime.date.fromtimestamp(period_params.to),
            adjustment=Adjustment.ALL,
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
