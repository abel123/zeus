from enum import Enum
from typing import Any, Dict, List, Optional, Protocol, LiteralString, Union

from pydantic import BaseModel, Field, RootModel
from pydantic.dataclasses import dataclass as pyd_dataclass

Market = {
    "china": {"session": "0930-1131,1300-1501", "timezone": "Asia/Shanghai"},
    "hongkong": {"session": "0930-1200,1300-1601", "timezone": "Asia/Shanghai"},
    "america": {
        "session": "0900-1631",
        "timezone": "America/New_York",
    },
}


class Exchange(BaseModel):
    name: str
    value: str
    desc: str


class SymbolType(BaseModel):
    name: str
    value: str


class Unit(BaseModel):
    id: str
    name: str
    description: str


class Config(BaseModel):
    exchanges: Optional[List[Exchange]] = None
    supported_resolutions: Optional[List[str]] = None
    units: Optional[Dict[str, List[Unit]]] = None
    currency_codes: Optional[List[str]] = None
    supports_marks: bool = False
    supports_time: bool = False
    supports_timescale_marks: bool = False
    symbols_types: Optional[List[SymbolType]] = None
    supports_search: bool = False
    supports_group_request: bool = False


class SeriesFormat(str, Enum):
    price = "price"
    volume = "volume"


class DataStatus(str, Enum):
    streaming = "streaming"
    endofday = "endofday"
    pulsed = "pulsed"
    delayed_streaming = "delayed_streaming"


class LibrarySymbolInfo(BaseModel):
    name: str
    full_name: str
    base_name: Optional[List[str]] = None
    ticker: Optional[str] = None
    description: str
    type: str
    session: str
    session_display: Optional[str] = None
    holidays: Optional[str] = None
    corrections: Optional[str] = None
    exchange: str
    listed_exchange: str
    timezone: str  # TODO: Timezone type
    format: SeriesFormat
    pricescale: float
    minmov: int
    fractional: Optional[bool] = None
    minmove2: int
    has_intraday: Optional[bool] = None
    supported_resolutions: List[str]
    intraday_multipliers: Optional[List[str]] = None
    has_seconds: Optional[bool] = None
    has_ticks: Optional[bool] = None
    seconds_multipliers: Optional[List[str]] = None
    has_daily: Optional[bool] = None
    has_weekly_and_monthly: Optional[bool] = None
    has_empty_bars: Optional[bool] = None
    has_no_volume: Optional[bool] = None
    volume_precision: Optional[int] = None
    data_status: Optional[DataStatus] = None
    expired: Optional[bool] = None
    expiration_date: Optional[int] = None
    sector: Optional[str] = None
    industry: Optional[str] = None
    currency_code: Optional[str] = None
    original_currency_code: Optional[str] = None
    unit_id: Optional[str] = None
    original_unit_id: Optional[str] = None
    unit_conversion_types: Optional[List[str]] = None


class SearchSymbolResultItem(BaseModel):
    symbol: str
    full_name: str
    description: str
    exchange: str
    ticker: str
    type: str


class Bar(BaseModel):
    time: int
    close: float
    open: float
    high: float
    low: float
    volume: int


class HistoryPartialDataResponse(BaseModel):
    s: str
    time: List[int]
    close: List[float]
    open: Optional[List[float]] = []
    high: Optional[List[float]] = []
    low: Optional[List[float]] = []
    volume: Optional[List[int]] = []


class TimescaleMarkColor(BaseModel):
    red: str = "red"
    green: str = "green"
    blue: str = "blue"
    yellow: str = "yellow"


class TimescaleMark(BaseModel):
    id: Union[str, int]
    time: int
    color: TimescaleMarkColor
    label: str
    tooltip: List[str]


class PeriodParams(BaseModel):
    from_: int = Field(alias="from")
    to: int
    countBack: int | None = None
    firstDataRequest: bool


class MacdConfig(BaseModel):
    fast: int
    slow: int
    signal: int
    source: str = "close"

    def __hash__(self) -> int:
        return hash(hash(self.fast + self.slow + self.signal) + hash(self.source))


class RequestParam(BaseModel):
    from_: int = Field(alias="from")
    to: int
    symbol: str
    resolution: str
    macd_config: List[MacdConfig]
