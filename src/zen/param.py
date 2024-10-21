from pydantic import BaseModel, Field


class ZenElementRequest(BaseModel):
    from_: int = Field(alias="from")
    resolution: str
    symbol: str
    to: int
