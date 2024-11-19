"""
document: https://www.tradingview.com/script/7VXF3amy-Multi-Timeframe-TTM-Squeeze-Pro/

"""

from datetime import datetime, timedelta
import gc
import os
import time
from loguru import logger
from broker.enums import Resolution
from broker.mixed import Mixed
from ib_insync import util
import pandas_ta as ta
from colored import Back, Fore, Style
from curd import update_ttm
from pandas_ta.momentum import bias


async def excute(lines, nofilter, latest=False, reverse=False, sort="sum"):
    logger.debug("args {} {}", nofilter, latest)
    gc.disable()

    os.environ["ID"] = "233"

    broker = Mixed()
    result = []
    rows = []

    for idx, l in enumerate(lines[1:]):
        l = l.split()
        logger.info(f"{idx} {l}")
        info = {}
        for freq in [Resolution.Day, Resolution.Hour, Resolution.Min15]:
            if latest:
                time.sleep(1)
            last = datetime.now().timestamp() - timedelta(minutes=1).total_seconds()
            listener = await broker.subscribe(
                f":{l[0]}",
                freq,
                int(last - Mixed.offset[freq].total_seconds()),
                int(last),
                -1 if latest else None,
                False if latest else True,
            )

            # logger.debug("bars {}", listener.bars)
            df = util.df(listener.bars[max(len(listener.bars) - 100, 0) :])
            out = df.ta.squeeze_pro(
                high=df["high"],
                low=df["low"],
                close=df["close"],
                bb_length=20,
                kc_length=20,
                use_tr=True,
                append=True,
            )

            ba = bias(df["close"], 5)
            wide = out["SQZPRO_ON_WIDE"][::-1].tolist().index(0)
            normal = out["SQZPRO_ON_NORMAL"][::-1].tolist().index(0)
            narrow = out["SQZPRO_ON_NARROW"][::-1].tolist().index(0)

            info[str(freq.value)] = {
                "wide_count": wide,
                "normal_count": normal,
                "narrow_count": narrow,
                "bias": ba.to_list(),
                "wide_index": (
                    out["SQZPRO_ON_WIDE"][::-1][:10].tolist().index(1)
                    if 1 in out["SQZPRO_ON_WIDE"][::-1][:10].tolist()
                    else 11
                ),
                "normal_index": (
                    out["SQZPRO_ON_NORMAL"][::-1][:10].tolist().index(1)
                    if 1 in out["SQZPRO_ON_NORMAL"][::-1][:10].tolist()
                    else 11
                ),
                "narrow_index": (
                    out["SQZPRO_ON_NARROW"][::-1][:10].tolist().index(1)
                    if 1 in out["SQZPRO_ON_NARROW"][::-1][:10].tolist()
                    else 11
                ),
                "SQZPRO_ON_WIDE": out["SQZPRO_ON_WIDE"].tolist(),
                "SQZPRO_ON_NORMAL": out["SQZPRO_ON_NORMAL"].tolist(),
                "SQZPRO_ON_NARROW": out["SQZPRO_ON_NARROW"].tolist(),
                "SQZPRO_20_2.0_20_2_1.5_1": out["SQZPRO_20_2.0_20_2_1.5_1"].tolist(),
                "SQZPRO_NO": out["SQZPRO_NO"].tolist(),
                "SQZPRO_OFF": out["SQZPRO_OFF"].tolist(),
            }

        rows.append((l[0], info))
    # asyncio.run(update_ttm(rows))

    rows.sort(key=lambda f: score(f, True, reverse, sort))
    count = 0
    for symbol, r in rows:
        if (
            sum(r["1D"]["SQZPRO_ON_WIDE"][-8:]) > 5
            and r["1D"]["SQZPRO_ON_WIDE"][-2] == 1
        ) or nofilter:
            print(
                "{:<3}{:<5} {}: 1D - {:<12}: {} |  1h - {} | 15 - {}".format(
                    count,
                    symbol,
                    fmt_list(r["1D"]["bias"][-3:]),
                    f"{r["1D"]["wide_count"]}, {r["1D"]["normal_count"]}, {r["1D"]["narrow_count"]}",
                    f"{color_line(r["1D"], 10)}",
                    f"{color_line(r["60"],18)}",
                    f"{color_line(r["15"],15)}",
                )
            )

            count += 1


def fmt_list(l):
    str = Style.reset + Fore.red + "↑" + Style.reset
    if sum(l) < 0:
        str = Style.reset + Fore.green + "↓" + Style.reset

    return str + ",".join(["{:>5.1f}".format(x * 100) for x in l])


def color_line(data, len=5):
    str = ""
    if False:
        str += "{} {} {} ".format(
            -data["narrow_index"],
            -data["normal_index"],
            -data["wide_index"],
        )
        """str += "{} ".format(
            (
                data["SQZPRO_ON_WIDE"][-5:],
                data["SQZPRO_ON_NORMAL"][-5:],
                data["SQZPRO_ON_NARROW"][-5:],
            )
        )"""
    for i in range(1, len + 1)[::-1]:
        count = (
            data["SQZPRO_ON_WIDE"][-i]
            + data["SQZPRO_ON_NORMAL"][-i]
            + data["SQZPRO_ON_NARROW"][-i]
        )
        if count == 0:
            str += Style.reset + Fore.grey_46 + Back.green + "0" + Style.reset
        elif count == 1:
            str += Style.reset + Fore.grey_46 + Back.black + "1" + Style.reset
        elif count == 2:
            str += Style.reset + Fore.grey_46 + Back.red + "2" + Style.reset
        elif count == 3:
            str += Style.reset + Fore.grey_46 + Back.orange_3 + "3" + Style.reset
    return str


def score(info, yes, reverse, sort):
    last_cnt = 8
    from builtins import sum

    if sort == "sum":
        return (
            (
                sum(info[1]["1D"]["bias"][-3:]) < 0
                if reverse
                else sum(info[1]["1D"]["bias"][-3:]) > 0
            ),
            sum(
                info[1]["1D"]["SQZPRO_ON_WIDE"][-last_cnt:]
                + info[1]["1D"]["SQZPRO_ON_NORMAL"][-last_cnt:]
                + info[1]["1D"]["SQZPRO_ON_NARROW"][-last_cnt:]
            ),
        )
    elif sort == "diff":
        return (
            (
                (
                    info[1]["1D"]["SQZPRO_ON_WIDE"][-1]
                    + info[1]["1D"]["SQZPRO_ON_NORMAL"][-1]
                    + info[1]["1D"]["SQZPRO_ON_NARROW"][-1]
                )
                - (
                    info[1]["1D"]["SQZPRO_ON_WIDE"][-2]
                    + info[1]["1D"]["SQZPRO_ON_NORMAL"][-2]
                    + info[1]["1D"]["SQZPRO_ON_NARROW"][-2]
                ),
                (
                    info[1]["1D"]["SQZPRO_ON_WIDE"][-1]
                    + info[1]["1D"]["SQZPRO_ON_NORMAL"][-1]
                    + info[1]["1D"]["SQZPRO_ON_NARROW"][-1]
                ),
            )
            if reverse
            else (
                (
                    info[1]["1D"]["SQZPRO_ON_WIDE"][-2]
                    + info[1]["1D"]["SQZPRO_ON_NORMAL"][-2]
                    + info[1]["1D"]["SQZPRO_ON_NARROW"][-2]
                )
                - (
                    info[1]["1D"]["SQZPRO_ON_WIDE"][-1]
                    + info[1]["1D"]["SQZPRO_ON_NORMAL"][-1]
                    + info[1]["1D"]["SQZPRO_ON_NARROW"][-1]
                ),
                (
                    info[1]["1D"]["SQZPRO_ON_WIDE"][-1]
                    + info[1]["1D"]["SQZPRO_ON_NORMAL"][-1]
                    + info[1]["1D"]["SQZPRO_ON_NARROW"][-1]
                ),
            )
        )
    return (
        -info[1]["1D"]["narrow_index"],
        -info[1]["1D"]["normal_index"],
        -info[1]["1D"]["wide_index"],
    )
    seq = []
    len = 5
    sum = 0
    for i in range(1, len + 1)[::-1]:
        count = (
            info[1]["1D"]["SQZPRO_ON_WIDE"][-i]
            + info[1]["1D"]["SQZPRO_ON_NORMAL"][-i]
            + info[1]["1D"]["SQZPRO_ON_NARROW"][-i]
        )
        if count == 0 and yes == False:
            count = 4
        seq.append(count)

    for i in range(1, 5):
        sum += i * abs(seq[i] - seq[i - 1])
    return sum
