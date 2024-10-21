from enum import Enum


class Resolution(Enum):
    Min = "1"
    Min3 = "3"
    Min5 = "5"
    minute_10 = "10"
    Min15 = "15"
    Min30 = "30"
    Hour = "60"
    day = "1D"
    Week = "1W"
    Month = "1M"
    year = "12M"
