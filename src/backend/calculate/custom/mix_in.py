import asyncio
from datetime import datetime
from typing import Callable, Dict, List, Protocol, Tuple
from cachetools import LRUCache

from ib_insync import BarData
from loguru import logger
import pytz
from backend.broker.ib.options import get_tsla_option_list
from backend.calculate.protocol import Processor, Symbol, WatcherProtocol
from backend.utils.convert import local_time
from czsc.analyze import CZSC
from czsc.enum import Freq
from czsc.objects import RawBar, Signal


class ContractSignals(WatcherProtocol):
    def __init__(
        self,
        symbol: Symbol,
        freq: Freq,
        processors: List[Processor],
        matcher: Callable[[List[Signal]], None] = None,
    ) -> None:
        super().__init__()
        self.symbol: Symbol = symbol
        self.freq = freq
        self.czsc: CZSC = None
        self.processors = processors
        self.matcher = matcher
        self.latest_dt = datetime.now()
        self.reset(self.symbol.raw, self.freq)

    def id(self) -> str:
        return f"{__name__}_{self.__class__.__name__}:{self.symbol.raw}-{self.freq}"

    def contracts(self) -> List[Tuple[Symbol, Freq]]:
        return [(self.symbol, self.freq)]

    def on_bar_update(self, bar: RawBar, new_bar: bool, skip_event: bool = False):
        try:
            events: List[Signal] = []

            self.czsc.update(bar)
            if skip_event:
                return None

            for p in self.processors:
                ev = p.process(self.czsc, new_bar)
                if ev is not None:
                    if isinstance(ev, Signal):
                        events.append(ev)
                    else:
                        events.extend(ev)

            if self.matcher is not None:
                self.matcher(events, self.latest_dt)

            return events
        except Exception as e:
            logger.exception(f"except {e}")

    def reset(self, symbol_raw: str, freq: Freq) -> None:
        logger.warning(f"reset {symbol_raw} {freq}")
        self.czsc: CZSC = CZSC(
            self.symbol,
            self.freq,
            bars=[],
        )
        for p in self.processors:
            p.reset()


class MultipleContractSignals(WatcherProtocol):
    def __init__(
        self,
        contract_singals: List[ContractSignals],
        matcher: Callable[[List[Signal]], None] = None,
    ) -> None:
        super().__init__()
        self.contract_signals = contract_singals
        self.contract_to_signals: Dict[Tuple[str, Freq], ContractSignals] = dict()
        for cs in self.contract_signals:
            self.contract_to_signals[(cs.symbol.raw, cs.freq)] = cs
        self.matcher = matcher
        self.latest_dt = datetime.fromtimestamp(0, pytz.UTC)

    def id(self) -> str:
        id = ""
        for cs in self.contract_signals:
            id = id + f"{cs.symbol}-{cs.freq.value} "
        return f"{__name__}_{self.__class__.__name__}:[ {id}] "

    def contracts(self) -> List[Tuple[Symbol, Freq]]:
        result = []
        for cs in self.contract_signals:
            result.extend(cs.contracts())
        return result

    def on_bar_update(self, bar: RawBar, new_bar: bool, skip_event: bool = False):
        if self.contract_to_signals.get((bar.symbol, bar.freq), None) == None:
            if bar.symbol.startswith("TSLA "):  # option
                ...
            # logger.warning(f"{bar.symbol}-{bar.freq} has no related signal processor")
            return

        cs = self.contract_to_signals.get((bar.symbol, bar.freq))
        self.latest_dt = max(self.latest_dt, bar.dt)

        events: List[Signal] = []
        evs = cs.on_bar_update(bar, new_bar)
        if evs is not None:
            events.extend(evs)

        if bar.freq == Freq.F1 and bar.symbol == "TSLA":
            self.tsla_price = bar.close
            # contracts = asyncio.get_event_loop().run_until_complete(
            #    get_tsla_option_list()
            # )
            # logger.debug(f"contracts: {contracts[:5]}")

        for k, v in self.contract_to_signals.items():
            if (k[0] != bar.symbol or k[1] != bar.freq) and len(v.czsc.bars_raw) > 0:
                evs = v.on_bar_update(v.czsc.bars_raw[-1], False, False)
                if evs is not None:
                    events.extend(evs)
                False and logger.debug(
                    f"process {local_time(self.latest_dt)} events {k}, {events}"
                )

        if True and len(events) > 4:
            logger.warning(f"{local_time(bar.dt)} generated events {events}")

        if self.matcher is not None:
            self.matcher(events, self.latest_dt, bar.symbol)
        return events

    def reset(self, symbol_raw: str, freq: Freq):
        logger.warning(f"reset {self.id()} {symbol_raw} {freq}")
        for k, v in self.contract_to_signals.items():
            if k[0] == symbol_raw and k[1] == freq:
                v.reset(symbol_raw, freq)
