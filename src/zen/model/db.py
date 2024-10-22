from sqlmodel import Field, SQLModel


class Symbols(SQLModel, table=True):
    screener: str = Field(default=None, primary_key=True)
    type: str
    pricescale: int
    exchange: str
    symbol: str = Field(default=None, primary_key=True)
    logoid: str
    desc: str
