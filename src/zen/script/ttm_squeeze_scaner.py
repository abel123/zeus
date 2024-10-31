"""
document: https://www.tradingview.com/script/7VXF3amy-Multi-Timeframe-TTM-Squeeze-Pro/

"""

import asyncio
from loguru import logger
from broker.enums import Resolution
from broker.mixed import Mixed
from ib_insync import util
import pandas_ta as ta


def excute(lines):
    broker = Mixed()
    result = []

    for idx, l in enumerate(lines[1:]):
        l = l.split()
        logger.debug(f"{idx} {l}")
        for freq in [
            Resolution.Day,
            # Resolution.Hour,
        ]:
            listener = asyncio.run(broker.subscribe(f":{l[0]}", freq, 0, 0, None, True))
            df = util.df(listener.bars)
            out = df.ta.squeeze_pro(
                high=df["high"],
                low=df["low"],
                close=df["close"],
                bb_length=20,
                kc_length=20,
                use_tr=True,
                append=True,
            )

            wide = out["SQZPRO_ON_WIDE"][::-1].tolist().index(0)
            normal = out["SQZPRO_ON_NORMAL"][::-1].tolist().index(0)
            narrow = out["SQZPRO_ON_NARROW"][::-1].tolist().index(0)
            if wide >= 5:
                result.append((narrow, normal, wide, l[0]))
        result.sort()
    result = [print(idx, tuple(reversed(r))) for idx, r in enumerate(result)]
    # print(result)
