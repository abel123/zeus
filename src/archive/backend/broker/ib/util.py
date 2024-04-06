from datetime import timedelta
import math

from ib_insync import Contract


def timedelta_to_duration_str(duration: timedelta) -> str:
    if duration.days >= 360:
        return f"{math.ceil(duration.total_seconds()*1.0 / timedelta(365).total_seconds()):.0f} Y"
    elif duration.days >= 30:
        return f"{math.ceil(duration.total_seconds()*1.0 / timedelta(30).total_seconds()):.0f} M"
    elif duration.days >= 7:
        return f"{math.ceil(duration.total_seconds()*1.0 / timedelta(7).total_seconds()):.0f} W"
    elif duration.days >= 1:
        return f"{math.ceil(duration.total_seconds()*1.0/timedelta(1).total_seconds()):.0f} D"
    else:
        return f"{max(30, duration.total_seconds()):.0f} S"
