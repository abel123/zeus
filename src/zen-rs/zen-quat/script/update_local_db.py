from datetime import date, datetime
import sqlite3
import sys
from typing import List, Union
import ib_insync
from pyrate_limiter import Duration, Limiter, Rate, InMemoryBucket
import pytz
import logger
from loguru import logger


logger.add("log/local_db.txt", rotation="500 MB")

# sync ib
f = open("./script/watchlist.txt", "r")

ib = ib_insync.IB()
ib.connect("127.0.0.1", 14001, clientId=1999, timeout=10)

limiter = Limiter(
    Rate(50, Duration.SECOND),
    raise_when_fail=False,  # Default = True
    max_delay=1000,  # Default = None
)
decorator = limiter.as_decorator()


def mapping(*args, **kwargs):
    return "demo", 1


@decorator(mapping)
def reqHistoricalData(
    contract: ib_insync.Contract,
    endDateTime: Union[datetime, date, str, None],
    durationStr: str,
    barSizeSetting: str,
    whatToShow: str,
    useRTH: bool,
    formatDate: int = 1,
    keepUpToDate: bool = False,
    chartOptions: List[ib_insync.TagValue] = [],
    timeout: float = 60,
) -> ib_insync.BarDataList:
    return ib.reqHistoricalData(
        contract,
        endDateTime,
        durationStr,
        barSizeSetting,
        whatToShow,
        useRTH,
        formatDate,
        keepUpToDate,
    )


for line in f.readlines():
    if line.startswith("."):
        continue
    symbol, _, exchange = line.split()[:3]
    if exchange == "美股":
        with sqlite3.connect("tradingview.db", isolation_level=None) as con:
            # con.set_trace_callback(lambda x: logger.debug(x))

            con.execute("DELETE from bar_history where symbol = ?", [symbol])
            # '60 S', '30 D', '13 W', '6 M', '10 Y'.
            for bar_size, duration in {
                "1 min": "2 D",
                "3 mins": "7 D",
                "5 mins": "2 M",
                "15 mins": "3 M",
                "1 hour": "4 M",
                "1 day": "3 Y",
                "1 week": "4 Y",
            }.items():
                logger.debug(
                    "update symbol {} {}: {} - {}", exchange, symbol, bar_size, duration
                )
                end_date = datetime.now()
                bars = reqHistoricalData(
                    ib_insync.Stock(symbol=symbol, exchange="SMART", currency="USD"),
                    end_date,
                    duration,
                    bar_size,
                    whatToShow="TRADES",
                    useRTH=True,
                    formatDate=2,
                    keepUpToDate=False,
                )

                freq_mapping = {
                    "1 min": "F1",
                    "3 mins": "F3",
                    "5 mins": "F5",
                    "15 mins": "F15",
                    "1 hour": "F60",
                    "1 day": "D",
                    "1 week": "W",
                }
                con.executemany(
                    "INSERT INTO bar_history(symbol, freq, dt, high, low, open, close, volume) values(?,?,?,?,?,?,?,?)",
                    [
                        (
                            symbol,
                            freq_mapping.get(bar_size, "unknown"),
                            int(
                                bar.date.timestamp()
                                if hasattr(bar.date, "timestamp")
                                else datetime(
                                    year=bar.date.year,
                                    month=bar.date.month,
                                    day=bar.date.day,
                                    tzinfo=pytz.timezone("America/New_York"),
                                ).timestamp()
                            ),
                            bar.high,
                            bar.low,
                            bar.open,
                            bar.close,
                            bar.volume,
                        )
                        for bar in bars
                    ],
                )
        # break
