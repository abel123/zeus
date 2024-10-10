from ib_insync import *

ib = IB()
ib.connect("127.0.0.1", 14001, clientId=1)

summary = ib.accountSummary()
print(summary)
