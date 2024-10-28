import zen_core

from broker import ib
from broker.enums import Resolution


class Mixed:
    def __init__(self):
        self.ib = ib.Broker()

    def __del__(self):
        self.ib.ib.disconnect()

    async def subscribe(
        self,
        symbol: str,
        freq: Resolution,
        from_: int,
        to: int,
        cout_back: int | None,
        local: bool,
    ):
        [exchange, symbol] = symbol.split(":")[:2]

        listener = await self.ib.subscribe(
            symbol=symbol, freq=freq, from_=from_, to=to, non_realtime=False
        )
        return listener
