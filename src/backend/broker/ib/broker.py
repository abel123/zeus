import calendar
from collections import OrderedDict
from datetime import datetime, timedelta, timezone
from typing import Any, List
from cachetools import LRUCache
import ib_insync
from loguru import logger
from backend.broker.ib.signals import Watcher
from backend.broker.ib.subscribe_manager import SubscribeManager
from backend.broker.ib.util import timedelta_to_duration_str
from backend.calculate.protocol import Symbol, SymbolType
from backend.calculate.zen.signal.macd import Config

from backend.datafeed.tv_model import Bar, LibrarySymbolInfo, MacdConfig, PeriodParams
from backend.utils.magic import SingletonABCMeta
from czsc.enum import Freq
from asyncache import cached


class Broker(object):
    __metaclass__ = SingletonABCMeta

    freq_map = {
        "1 day": Freq.D,
        "1 month": Freq.M,
        "1 week": Freq.W,
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

    last_macd_config = OrderedDict()

    @cached(LRUCache(1024))
    async def get_head_time(symbol_info: LibrarySymbolInfo) -> int:
        contract = ib_insync.Stock(
            symbol_info.name,
            "SMART",
            "USD",
            primaryExchange="NASDAQ"
            if symbol_info.exchange == ""
            else symbol_info.exchange,
        )
        ts = await Broker.ib.reqHeadTimeStampAsync(contract, "TRADES", True, 2)
        return ts

    async def get_bars(
        symbol_info: LibrarySymbolInfo,
        resolution: str,
        period_params: PeriodParams,
        macd_config: List[MacdConfig],
    ) -> (List[Bar], Watcher):
        """
         barSizeSetting: Time period of one bar. Must be one of:
                '1 secs', '5 secs', '10 secs' 15 secs', '30 secs',
                '1 min', '2 mins', '3 mins', '5 mins', '10 mins', '15 mins',
                '20 mins', '30 mins',
                '1 hour', '2 hours', '3 hours', '4 hours', '8 hours',
                '1 day', '1 week', '1 month'.
        }
        """
        mapping = {
            "1D": "1 day",
            "1M": "1 month",
            "1W": "1 week",
            "240": "4 hours",
            "180": "2 hours",
            "120": "2 hours",
            "60": "1 hour",
            "30": "30 mins",
            "20": "20 mins",
            "15": "15 mins",
            "10": "10 mins",
            "5": "5 mins",
            "3": "3 mins",
            "2": "2 mins",
            "1": "1 min",
        }
        freq_map = {
            "1D": Freq.D,
            "1M": Freq.M,
            "1W": Freq.W,
            "240": Freq.F240,
            "480": Freq.F480,
            "120": Freq.F120,
            "60": Freq.F60,
            "30": Freq.F30,
            "20": Freq.F20,
            "15": Freq.F15,
            "10": Freq.F10,
            "5": Freq.F5,
            "3": Freq.F3,
            "2": Freq.F2,
            "1": Freq.F1,
        }
        try:
            symbol = Symbol(
                raw=symbol_info.name,
                exchange=symbol_info.exchange,
                type=SymbolType.OPTION
                if symbol_info.exchange == "option"
                else SymbolType.STOCK,
            )

            # [contract] = await Broker.ib.qualifyContractsAsync(contract)
            freq = freq_map.get(resolution)
            use_cache = False
            if len(macd_config) != 0:
                if set(macd_config) <= Broker.last_macd_config.get(
                    (symbol.raw, resolution), set()
                ):
                    use_cache = True
                else:
                    use_cache = False
                    Broker.last_macd_config[(symbol.raw, resolution)] = set(macd_config)

            watcher = None
            id = Watcher(
                [],
                symbol,
                freq,
                reset=False,
            ).id()

            logger.debug(
                {
                    "contract ": symbol.raw,
                    "connect_state": SubscribeManager().ib.client.connState,
                    "use_cache": use_cache,
                    "peroid": period_params,
                    "config": macd_config,
                    "id": id,
                    "freq": freq,
                }
            )

            ib_bars, new_subscriber = await SubscribeManager().subscribe(
                symbol,
                freq,
                period_params.from_,
                period_params.to,
                period_params.countBack,
            )
            logger.debug(
                {
                    "new_subscriber": new_subscriber,
                }
            )

            if (new_subscriber or use_cache == False) and len(macd_config) > 0:
                watcher = Watcher(
                    SubscribeManager().raw_bars(symbol, freq),
                    symbol,
                    freq,
                    Config(macd_config=macd_config),
                )
                SubscribeManager().remove_watcher(watcher)
                logger.warning(f"add watcher {id} {freq} {watcher}")
                SubscribeManager().upsert_watcher(watcher)
            elif len(macd_config) > 0:
                watcher = SubscribeManager().get_watcher(
                    symbol,
                    freq,
                    id,
                )
            # df = ib_insync.util.df(ib_bars[:5] + ib_bars[-10:])
            # df is not None and not df.empty and logger.debug(f"ib bars:\n {df}")
            return [
                Bar(
                    time=bar.date.timestamp()
                    if isinstance(bar.date, datetime)
                    else datetime.fromtimestamp(
                        calendar.timegm(bar.date.timetuple())
                    ).timestamp()
                    + timedelta(hours=5).total_seconds(),
                    close=bar.close,
                    open=bar.open,
                    high=bar.high,
                    low=bar.low,
                    volume=bar.volume,
                )
                for bar in ib_bars
            ], watcher
        except Exception as e:
            logger.exception(f"exception ==========")
            return None, None
