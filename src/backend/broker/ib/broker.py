import calendar
from collections import OrderedDict
from datetime import date, datetime, timedelta, timezone
import math
from typing import Any, List
from cachetools import LRUCache, TLRUCache, TTLCache
from ib_insync import IB, Contract
import ib_insync
from loguru import logger
import pytz
from backend.broker.ib.signals import Watcher
from backend.broker.ib.subscribe_manager import SubscribeManager
from backend.broker.ib.util import get_symbol, timedelta_to_duration_str
from backend.calculate.zen import signal
from backend.calculate.zen.signal.macd import Config, MACDArea

from backend.datafeed.tv_model import Bar, LibrarySymbolInfo, MacdConfig, PeriodParams
from backend.utils.magic import SingletonABCMeta
from czsc.analyze import CZSC
from czsc.enum import Freq
from czsc.objects import RawBar
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
            contract = ib_insync.Stock(
                symbol_info.name,
                "SMART",
                "USD",
                primaryExchange="NASDAQ"
                if symbol_info.exchange == ""
                else symbol_info.exchange,
            )

            if symbol_info.exchange == "option":
                contract = ib_insync.Option(
                    exchange="SMART",
                    localSymbol=symbol_info.name,
                    primaryExchange="CBOE",
                )
            # [contract] = await Broker.ib.qualifyContractsAsync(contract)

            use_cache = False
            if len(macd_config) != 0:
                if set(macd_config) <= Broker.last_macd_config.get(resolution, set()):
                    use_cache = True
                else:
                    use_cache = False
                    Broker.last_macd_config[resolution] = set(macd_config)

            barSize = mapping.get(resolution)

            watcher = None
            id = Watcher(
                [],
                get_symbol(contract),
                freq_map.get(resolution),
                reset=False,
            ).id()

            ib_bars, new_subscriber = await SubscribeManager().subscribe(
                contract,
                barSize,
                period_params.from_,
                period_params.to,
                period_params.countBack,
            )

            logger.debug(
                {
                    "contract ": contract.symbol + contract.localSymbol,
                    "connect_state": SubscribeManager().ib.client.connState,
                    "use_cache": use_cache,
                    "peroid": period_params,
                    "config": macd_config,
                    "new_subscriber": new_subscriber,
                    "id": id,
                    "barSize": barSize,
                }
            )

            if (new_subscriber or use_cache == False) and len(macd_config) > 0:
                watcher = Watcher(
                    SubscribeManager().raw_bars(contract, barSize),
                    get_symbol(contract),
                    freq_map.get(resolution),
                    Config(macd_config=macd_config),
                )
                SubscribeManager().remove_watcher(contract, barSize, watcher)
                logger.warning(f"add watcher {id} {barSize} {watcher}")
                SubscribeManager().add_watcher(contract, barSize, watcher)
            elif len(macd_config) > 0:
                watcher = SubscribeManager().get_watcher(
                    contract,
                    barSize,
                    id,
                )
            # df = ib_insync.util.df(requester.bars[:5] + requester.bars[-6:])
            # df is not None and not df.empty and logger.debug(f"ib bars:\n {df}")
            return [
                Bar(
                    time=bar.date.timestamp()
                    if isinstance(bar.date, datetime)
                    else datetime.utcfromtimestamp(
                        calendar.timegm(bar.date.timetuple())
                    ).timestamp()
                    + timedelta(hours=1).total_seconds(),
                    # bar.date.timestamp() * 1000,
                    close=bar.close,
                    open=bar.open,
                    high=bar.high,
                    low=bar.low,
                    volume=bar.volume,
                )
                for bar in ib_bars
            ], watcher
        except Exception as e:
            logger.warning(f"exception =========={e}")
            raise e
            return None, None
