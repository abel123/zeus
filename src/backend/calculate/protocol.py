from abc import abstractmethod
from typing import List, Protocol

from czsc.objects import RawBar


class WatcherProtocol(Protocol):
    @abstractmethod
    def id() -> str:
        ...

    @abstractmethod
    def on_bar_update(bar: RawBar, hasNewBar):
        ...

    @abstractmethod
    def reset():
        ...

    @abstractmethod
    def populate_bars(bars: List[RawBar]):
        ...
