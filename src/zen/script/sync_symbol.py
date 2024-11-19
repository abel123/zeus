import sqlite3
from moomoo import *

quote_ctx = OpenQuoteContext(host="127.0.0.1", port=9999)

ret, data = quote_ctx.get_plate_stock("HK.LIST1910")
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
            d[0],
            d[2],
            d[2],
        )
    )

print(rows[:5])
with sqlite3.connect("tradingview.db") as con:
    con.executemany("INSERT INTO symbols VALUES (?, ?, ?, ?, ?, ?, ?)", rows)


rows = []
for plate in ["SH.LIST3000005", "SH.LIST3000002"]:
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
                d[0].split(".")[0],
                d[0],
                d[2],
                d[2],
            )
        )

print(rows[:5])
with sqlite3.connect("tradingview.db") as con:
    con.executemany("INSERT INTO symbols VALUES (?, ?, ?, ?, ?, ?, ?)", rows)


ret, data = quote_ctx.get_plate_stock("US.USAALL")
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
            "america",
            "stock",
            100,
            "US",
            d[0],
            d[2],
            d[2],
        )
    )

print(rows[:5])
with sqlite3.connect("tradingview.db") as con:
    con.executemany("INSERT INTO symbols VALUES (?, ?, ?, ?, ?, ?, ?)", rows)

quote_ctx.close()
