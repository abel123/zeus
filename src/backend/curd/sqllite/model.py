from typing import List, LiteralString
from mayim import SQLiteExecutor, query
from pydantic import BaseModel
from sanic_ext import Extend


class Symbol(BaseModel):
    screener: str
    type: str
    pricescale: int
    exchange: str
    symbol: str
    logoid: str
    desc: str


class SymbolExecutor(SQLiteExecutor):
    @query(
        """SELECT * from symbols where screener=$screener AND (`symbol` LIKE $user_input or `desc` LIKE $user_input) LIMIT 100"""  # or desc like '%$user_input%')"
    )
    async def select_symbols(
        self,
        screener: LiteralString,
        type: LiteralString,
        user_input: LiteralString,
    ) -> List[Symbol]:
        ...

    @query("SELECT * from symbols WHERE screener=:screener AND `symbol`=:symbol")
    async def resolve_symbol(
        self, screener: LiteralString, type: LiteralString, symbol: LiteralString
    ) -> List[Symbol]:
        ...
