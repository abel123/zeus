from contextlib import asynccontextmanager
from typing import List, LiteralString
from sqlalchemy.ext.asyncio import AsyncSession, create_async_engine
from sqlalchemy.orm import sessionmaker
from sqlmodel import Field, Session, SQLModel, create_engine, or_, select

from model.db import Symbols

DATABASE_URL = "sqlite+aiosqlite:///tradingview.db"

# Create an asynchronous engine for the database
engine = create_async_engine(
    DATABASE_URL,
    echo=True,
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
                    Symbols.desc.like(f"%{user_input}%"),
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
            .where(Symbols.screener == screener)
            .where(Symbols.symbol == symbol)
            .limit(100)
        )

        return (await session.execute(statement)).first()
