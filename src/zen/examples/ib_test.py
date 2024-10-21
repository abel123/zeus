from datetime import datetime, timedelta, timezone
from ib_insync import *
import zen_core

ib = IB()

ib.connect(port=14001)

start_tm = datetime(2024, 10, 13)

stm = datetime.now()
bars = ib.reqHistoricalData(
    Stock("TSLA", "SMART", "USD"), datetime.now(), "2 Y", "1 hour", "TRADES", True
)

print(len(bars), bars[-1])
print(datetime.now() - stm)
stm = datetime.now()

z = zen_core.Zen("TSLA", zen_core.Freq.F1)
for b in bars:
    z.append(
        zen_core.Bar(
            b.date.replace(tzinfo=timezone(timedelta(hours=-4), "EST")),
            b.open,
            b.close,
            b.high,
            b.low,
            b.volume,
        )
        , True)
print(datetime.now() - stm, z.bi_info()[:2])
