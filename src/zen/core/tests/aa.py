from datetime import datetime, tzinfo, timezone, timedelta
from zoneinfo import ZoneInfo

import zen_core

a = zen_core.Bar(datetime.now(timezone(timedelta(hours=8), 'China')), 1, 2, 3, 4, 10000)
print(a)
