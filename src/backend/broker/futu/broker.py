# -*- coding: utf-8 -*-
import asyncio
from threading import Lock
from typing import List
from moomoo import *
import concurrent.futures
from loguru import logger
from backend.calculate.zen.signal.macd import Config, MACDArea
from backend.datafeed.tv_model import Bar, LibrarySymbolInfo, MacdConfig, PeriodParams
from czsc.analyze import CZSC
from czsc.enum import Freq
from czsc.objects import RawBar

futu_exec = concurrent.futures.ThreadPoolExecutor(1, thread_name_prefix="futu_exec")


class Broker:
    subscribed = dict()
    cache_lock = Lock()
    inited = False

    class CacheItem:
        def __init__(
            self,
            symbol,
            resolution,
            bars,
            macd_config=Config(macd_config=[MacdConfig(fast=12, slow=26, signal=9)]),
        ) -> None:
            self.macd_signal = MACDArea(macd_config, False)
            cl_freq_map = {
                "1": Freq.F1,
                "3": Freq.F3,
                "5": Freq.F5,
                "15": Freq.F15,
                "30": Freq.F30,
                "60": Freq.F60,
                "1D": Freq.D,
                "1W": Freq.W,
                "1M": Freq.M,
            }
            raw_bars = [
                RawBar(
                    symbol=symbol,
                    id=bar.time,
                    dt=datetime.fromtimestamp(bar.time),
                    freq=cl_freq_map.get(resolution, Freq.D),
                    open=bar.open,
                    close=bar.close,
                    high=bar.high,
                    low=bar.low,
                    vol=bar.volume,
                    amount=0.0,
                )
                for _, bar in enumerate(bars)
            ]
            self.czsc = CZSC(
                raw_bars,
                get_signals=self.macd_signal.macd_area_bc,
                on_bi_break=self.macd_signal.on_bi_break,
                on_bi_create=self.macd_signal.on_bi_create,
            )

    async def init():
        if Broker.inited == False:
            Broker.quote_ctx: OpenQuoteContext = (
                await asyncio.get_running_loop().run_in_executor(
                    futu_exec, lambda: OpenQuoteContext(host="127.0.0.1", port=11111)
                )
            )
            Broker.inited = True

    async def close():
        await asyncio.get_running_loop().run_in_executor(
            futu_exec, Broker.quote_ctx.close()
        )

    async def get_bars(
        symbol_info: LibrarySymbolInfo,
        resolution: str,
        period_params: PeriodParams,
        macd_config: List[MacdConfig],
    ) -> (List[Bar], CacheItem):
        await Broker.init()

        ctx: OpenQuoteContext = Broker.quote_ctx

        freq_map = {
            "1": KLType.K_1M,
            "3": KLType.K_3M,
            "5": KLType.K_5M,
            "15": KLType.K_15M,
            "30": KLType.K_30M,
            "60": KLType.K_60M,
            "1D": KLType.K_DAY,
            "1W": KLType.K_WEEK,
            "1M": KLType.K_MON,
        }
        freq_offset = {
            "1": timedelta(minutes=1),
            "3": timedelta(minutes=3),
            "5": timedelta(minutes=5),
            "15": timedelta(minutes=15),
            "30": timedelta(minutes=30),
            "60": timedelta(minutes=60),
            "1D": timedelta(hours=-10),
        }
        symbol = symbol_info.name
        if symbol_info.exchange == "HKEX":
            symbol = f"HK.{symbol}"
        elif symbol_info.exchange == "SSE":
            symbol = f"SH.{symbol}"
        elif symbol_info.exchange == "SZSE":
            symbol = f"SZ.{symbol}"

        ktype = freq_map.get(resolution, None)
        if ktype == None:
            return None, None

        if Broker.subscribed.get(f"{symbol}:{ktype}", None) == None:
            await asyncio.get_running_loop().run_in_executor(
                futu_exec,
                lambda: ctx.subscribe(symbol, [ktype], subscribe_push=True),
            )
            Broker.subscribed[f"{symbol}:{ktype}"] = True

        ret, data = await asyncio.get_running_loop().run_in_executor(
            futu_exec,
            lambda: ctx.get_cur_kline(
                symbol,
                min(
                    1000
                    if period_params.countBack is None
                    else period_params.countBack,
                    1000,
                ),
                freq_map[resolution],
                AuType.QFQ,
            ),
        )
        if ret != RET_OK:
            logger.error(f"get_cur_kline {symbol}-{ktype} error, {data}")
            del Broker.subscribed[f"{symbol}:{ktype}"]
            return None, None
        logger.debug(f"ret data\n {data.tail()}")
        bars = [
            Bar(
                time=datetime.fromisoformat(bar[2]).timestamp()
                - freq_offset.get(resolution, timedelta()).total_seconds(),
                open=bar[3],
                close=bar[4],
                high=bar[5],
                low=bar[6],
                volume=bar[7],
            )
            for bar in data.values.tolist()
        ]
        futu_bars = [bar for bar in bars if bar.time <= period_params.to]

        if len(macd_config) == 0:
            return futu_bars, None
        logger.debug(f"macd: {macd_config}")
        macd_signal = Config(macd_config=macd_config)
        item = Broker.CacheItem(symbol, resolution, bars, macd_signal)
        return futu_bars, item
