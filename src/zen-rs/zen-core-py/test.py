from zen_core_py import NewBar
import zen_core_py

a, b = zen_core_py.check_bi_py(
    [NewBar(123.1, 124.2, 125, 121, 123),
     NewBar(123.1, 124.2, 122, 120, 124),
     NewBar(123.1, 124.2, 125, 121, 125),
     NewBar(123.1, 124.2, 126, 122, 126),
     NewBar(123.1, 124.2, 127, 123, 127),
     NewBar(123.1, 124.2, 128, 124, 128),
     NewBar(123.1, 124.2, 127, 123, 128),
     ])
[print(x) for x in a]
print(b)
