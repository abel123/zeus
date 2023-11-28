import sqlite3
from moomoo import *

quote_ctx = OpenQuoteContext(host="127.0.0.1", port=11111)

ret, data = quote_ctx.get_plate_stock("HK.BK1910")
if ret == RET_OK:
    print(data)
    print(data["stock_name"][0])  # 取第一条的股票名称
    print(data["stock_name"].values.tolist())  # 转为 list
else:
    print("error:", data)
    os.abort()

rows = []
for d in data.values:
    rows.append(
        (
            "hongkong",
            "stock",
            100,
            "HKEX",
            d[0].replace("HK.", "HKEX:"),
            d[2],
            d[2],
        )
    )

print(rows[:5])
with sqlite3.connect("tradingview.db") as con:
    con.executemany("INSERT INTO symbols VALUES (?, ?, ?, ?, ?, ?, ?)", rows)


rows = []
for plate in ["SH.3000005", "SH.3000002"]:
    ret, data = quote_ctx.get_plate_stock(plate)
    if ret == RET_OK:
        print(data)
        print(data["stock_name"][0])  # 取第一条的股票名称
        print(data["stock_name"].values.tolist())  # 转为 list
    else:
        print("error:", data)
        os.abort()

    for d in data.values:
        rows.append(
            (
                "china",
                "stock",
                100,
                d[0].split(".")[0].replace("SH", "SSE").replace("SZ", "SZSE"),
                d[0].replace("SH.", "SSE:").replace("SZ.", "SZSE:"),
                d[2],
                d[2],
            )
        )

print(rows[:5])
with sqlite3.connect("tradingview.db") as con:
    con.executemany("INSERT INTO symbols VALUES (?, ?, ?, ?, ?, ?, ?)", rows)
quote_ctx.close()
