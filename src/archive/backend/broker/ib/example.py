from datetime import datetime
from typing import List
from ib_insync import *


# def get_tsla_option_list() -> List[Contract]:
# if True:
ib = IB()
ib.connect("127.0.0.1", 4001, clientId=12)

tsla = Stock("TSLA", "SMART", "USD")
ib.qualifyContracts(tsla)
ib.reqMarketDataType(4)
his = ib.reqHistoricalData(tsla, datetime.now(), "2 D", "1 hour", "TRADES", True, 2)
spxValue = his[-1].close
spxValue

chains = ib.reqSecDefOptParams(tsla.symbol, "", tsla.secType, tsla.conId)
util.df(chains)
print(chains)
chain = next(c for c in chains if c.tradingClass == "TSLA" and c.exchange == "SMART")

strikes = [
    strike
    for strike in chain.strikes
    if strike % 5 == 0 and spxValue - 10 < strike < spxValue + 10
]
expirations = sorted(exp for exp in chain.expirations)[:1]
rights = ["P", "C"]

contracts = [
    Option("TSLA", expiration, strike, right, "SMART", tradingClass="TSLA")
    for expiration in expirations
    for strike in strikes
    for right in rights
]

contracts = ib.qualifyContracts(*contracts)
ib.disconnect()
#    return contracts
