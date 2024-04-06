from datetime import datetime
from alpaca.data import CryptoHistoricalDataClient, StockHistoricalDataClient
from alpaca.data.requests import StockBarsRequest
from alpaca.data.timeframe import TimeFrame

# keys required
stock_client = StockHistoricalDataClient(
    "AKA93VVW12633ONJUZP9", "UkU9AuHtoTrQPrwa43iTAuEscX7JOHoxIGwGXuhc"
)

start = datetime(2022, 2, 1)
limit = 2000
request_params = StockBarsRequest(
    symbol_or_symbols="TSLA", timeframe=TimeFrame.Day, start=start, limit=limit
)


bars = stock_client.get_stock_bars(request_params)

# convert to dataframe
print(bars.df)
