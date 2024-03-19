import time
from moomoo import *


class CurKlineTest(CurKlineHandlerBase):
    def on_recv_rsp(self, rsp_pb):
        ret_code, data = super(CurKlineTest, self).on_recv_rsp(rsp_pb)
        if ret_code != RET_OK:
            print("CurKlineTest: error, msg: %s" % data)
            return RET_ERROR, data
        print("CurKlineTest ", data)  # CurKlineTest 自己的处理逻辑
        return RET_OK, data


quote_ctx = OpenQuoteContext(host="127.0.0.1", port=11111)
handler = CurKlineTest()
quote_ctx.set_handler(handler)  # 设置实时K线回调
ret, data = quote_ctx.subscribe(
    ["HK.00700"], [SubType.K_DAY]
)  # 订阅 K 线数据类型，OpenD 开始持续收到服务器的推送
if ret == RET_OK:
    print(data)
else:
    print("error:", data)

ret, data = quote_ctx.get_cur_kline(
    "HK.00700", 2, KLType.K_DAY, AuType.QFQ
)  # 获取港股00700最近2个 K 线数据
if ret == RET_OK:
    print(data)
    print(data["turnover_rate"][0])  # 取第一条的换手率
    print(data["turnover_rate"].values.tolist())  # 转为 list
else:
    print("error:", data)
time.sleep(15)  # 设置脚本接收 OpenD 的推送持续时间为15秒
quote_ctx.close()  # 关闭当条连接，OpenD 会在1分钟后自动取消相应股票相应类型的订阅
