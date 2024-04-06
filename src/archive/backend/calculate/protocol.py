from abc import abstractmethod
from enum import Enum
from typing import List, Protocol, Tuple

from pydantic import BaseModel
from czsc.analyze import CZSC
from czsc.enum import Freq

from czsc.objects import RawBar


class SymbolType(Enum):
    STOCK = "stock"
    OPTION = "option"


class Symbol(BaseModel):
    exchange: str = ""
    raw: str
    type: SymbolType

    def __str__(self) -> str:
        return f"{self.raw}"


class Processor(Protocol):
    @abstractmethod
    def process(self, czsc: CZSC, new_bar: bool):
        ...

    @abstractmethod
    def reset(self):
        ...


class WatcherProtocol(Protocol):
    @abstractmethod
    def id(self) -> str:
        ...

    @abstractmethod
    def on_bar_update(self, bar: RawBar, new_bar: bool, skip_event: bool = False):
        ...

    @abstractmethod
    def reset(self, symbol_raw: str, freq: Freq):
        ...

    @abstractmethod
    def contracts(self) -> List[Tuple[Symbol, Freq]]:
        ...
