from contextlib import asynccontextmanager
import json
from typing import List, LiteralString
from loguru import logger
from sqlalchemy.ext.asyncio import AsyncSession, create_async_engine
from sqlalchemy.orm import sessionmaker
from sqlmodel import (
    Field,
    Session,
    SQLModel,
    create_engine,
    delete,
    insert,
    or_,
    select,
)
from shelved_cache.decorators import asynccached
from shelved_cache import PersistentCache
from cachetools import LRUCache
from model.db import Symbols, Ttm_Info

DATABASE_URL = "sqlite+aiosqlite:///tradingview.db"

# Create an asynchronous engine for the database
engine = create_async_engine(
    DATABASE_URL,
    echo=False,
    future=True,
)


# Ayschronous Context manager for handling database sessions
@asynccontextmanager
async def get_session() -> AsyncSession:
    async_session = sessionmaker(engine, class_=AsyncSession, expire_on_commit=False)
    async with async_session() as session:
        yield session


async def select_symbols(
    screener: LiteralString,
    type: LiteralString,
    user_input: LiteralString,
) -> List[Symbols]:
    async with get_session() as session:
        statement = (
            select(Symbols)
            .where(Symbols.screener == screener)
            .where(
                or_(
                    Symbols.symbol.like(f"%{user_input}"),
                )
            )
            .limit(100)
        )

        return (await session.execute(statement)).fetchall()


async def resolve_symbols(
    screener: LiteralString, type: LiteralString, symbol: LiteralString
) -> Symbols | None:
    async with get_session() as session:
        statement = (
            select(Symbols)
            # .where(Symbols.screener == screener)
            .where(Symbols.symbol == symbol).limit(100)
        )

        return (await session.execute(statement)).first()


async def update_ttm(rows: list):
    async with get_session() as session:
        await session.execute(delete(Ttm_Info).where(1 == 1))
        logger.debug("rows {}", rows[0])

        for s, info in rows:
            session.add(Ttm_Info(symbol=s, info=json.dumps(info)))
        await session.commit()
