from datetime import datetime
import pytz


def local_time(dt: datetime):
    return dt.astimezone(pytz.timezone("Asia/Shanghai"))


def local_time_str(dt: datetime):
    return dt.astimezone(pytz.timezone("Asia/Shanghai")).strftime("%Y-%m-%d %H:%M:%S")
