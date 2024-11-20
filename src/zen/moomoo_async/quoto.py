import asyncio
from datetime import timezone

import ib_insync
import moomoo_async
from .client import Client
from moomoo.quote.quote_query import *

from moomoo.common import constant
from moomoo.common.pb import Qot_GetKL_pb2, Qot_UpdateKL_pb2


class Quoto(Client):
    def __init__(self, addr="127.0.0.1", port=9999):
        self.lock = asyncio.Lock()
        super().__init__(addr, port)
        self.subscribe_event += self.on_update

    async def _reconnect(self):
        async with self.lock:
            if self.connectState == Client.DISCONNECTED:
                await self.connect()

    async def subscribe(
        self,
        code_list,
        subtype_list,
        is_first_push=True,
        subscribe_push=True,
        is_detailed_orderbook=False,
        extended_time=False,
    ):
        await self._reconnect()
        kargs = {
            "code_list": code_list,
            "subtype_list": subtype_list,
            "conn_id": self.get_sync_conn_id(),
            "is_first_push": is_first_push,
            "subscribe_push": subscribe_push,
            "is_detailed_orderbook": is_detailed_orderbook,
            "extended_time": extended_time,
        }
        ret, msg, req_str = SubscriptionQuery.pack_subscribe_req(**kargs)
        if ret != constant.RET_OK:
            self._logger.warning("error {} {}", ret, msg)
            return ret, msg, None
        fut = self.startReq(self.unique_id)
        self.sendMsg(req_str)
        await fut
        rsp_pb = fut.result()
        if rsp_pb.retType != RET_OK:
            return RET_ERROR, rsp_pb.retMsg, None

        return RET_OK, "", None

    def adjust(self, ktype=KLType.K_DAY):
        res = {
            KLType.K_1M: timedelta(minutes=1),
            KLType.K_3M: timedelta(minutes=3),
            KLType.K_5M: timedelta(minutes=5),
            KLType.K_15M: timedelta(minutes=15),
            KLType.K_30M: timedelta(minutes=30),
            KLType.K_60M: timedelta(minutes=60),
            KLType.K_DAY: timedelta(hours=23),
        }.get(ktype, timedelta(seconds=0))
        # self._logger.debug("ktype {}, adjust {}", ktype, res)
        return res

    async def get_cur_kline(
        self,
        code,
        num,
        ktype=KLType.K_DAY,
        autype=AuType.QFQ,
        keep_update=False,
    ):
        kargs = {
            "code": code,
            "num": num,
            "ktype": ktype,
            "autype": autype,
            "conn_id": self.get_sync_conn_id(),
        }
        ret, msg, req_str = CurKlineQuery.pack_req(**kargs)
        if ret != constant.RET_OK:
            return ret, msg
        id = self.unique_id
        fut = self.startReq(self.unique_id)
        self.sendMsg(req_str)
        await fut
        rsp_pb: Qot_GetKL_pb2.Response = fut.result()
        if rsp_pb.retType != RET_OK:
            return RET_ERROR, rsp_pb.retMsg, None

        bars = ib_insync.BarDataList()
        bars.barSizeSetting = ktype
        bars.contract = ib_insync.Stock(symbol=code)
        bars.keepUpToDate = keep_update
        bars.reqId = id
        for bar in rsp_pb.s2c.klList:
            bars.append(self.convert(bar, ktype))

        if keep_update:
            self.start_keep_update(self.key(code, KLType.to_number(ktype)[1]), bars)
            # bars.updateEvent += lambda x, y: print(x, y)
        return RET_OK, "", bars

    async def unsubscribe(self, code_list, subtype_list, unsubscribe_all=False):
        kargs = {
            "code_list": code_list,
            "subtype_list": subtype_list,
            "unsubscribe_all": unsubscribe_all,
            "conn_id": self.get_sync_conn_id(),
        }

        ret, msg, req_str = SubscriptionQuery.pack_unsubscribe_req(**kargs)
        if ret != constant.RET_OK:
            return ret, msg
        fut = self.startReq(self.unique_id)
        self.sendMsg(req_str)
        await fut
        rsp_pb = fut.result()
        if rsp_pb.retType != RET_OK:
            self._logger.warning("unscribe {} {}", rsp_pb.retType, rsp_pb.retMsg)
            return RET_ERROR, rsp_pb.retMsg, None

        for code in code_list:
            for subtype in subtype_list:
                kline = {
                    SubType.K_1M: KLType.K_1M,
                    SubType.K_3M: KLType.K_3M,
                    SubType.K_5M: KLType.K_5M,
                    SubType.K_15M: KLType.K_15M,
                    SubType.K_60M: KLType.K_60M,
                    SubType.K_DAY: KLType.K_DAY,
                    SubType.K_WEEK: KLType.K_WEEK,
                }.get(subtype, None)
                if kline == None:
                    continue
                self.end_keep_update(self.key(code, KLType.to_number(kline)[1]))

        return RET_OK, "", None

    def start_keep_update(self, kline, container: ib_insync.BarDataList):
        self._logger.debug("kline {}, container {}", str(kline), len(container))
        self._results[kline] = container

    def end_keep_update(self, kline):
        self._results.pop(kline, None)

    def on_update(self, msg: Qot_UpdateKL_pb2.Response):
        sec = msg.s2c.security
        symbol = ".".join([Market.to_string(sec.market)[1], sec.code])
        ktype = msg.s2c.klType
        kline = self.key(symbol, msg.s2c.klType)

        ktype = KLType.to_string(ktype)[1]
        # self._logger.debug("kline {}", kline)

        container: ib_insync.BarDataList = self._results.get(kline, None)
        if len(msg.s2c.klList) == 0:
            return
        # self._logger.debug("msg {} {} {}", symbol, msg, len(container))

        if container is not None:
            if len(container) == 0:
                bar = msg.s2c.klList[0]
                container.append(self.convert(bar, ktype))
                container.updateEvent.emit(container, True)
                msg.s2c.klList = msg.s2c.klList[1:]

            for bar in msg.s2c.klList:
                if bar.timestamp > container[-1].date.timestamp():
                    container.append(self.convert(bar, ktype))
                    container.updateEvent.emit(container, True)

                elif bar.timestamp == container[-1].date.timestamp():
                    container[-1] = self.convert(bar, ktype)
                    container.updateEvent.emit(container, False)

    def convert(self, bar: Qot_Common_pb2.KLine, ktype: KLType):
        return ib_insync.BarData(
            date=datetime.fromtimestamp(
                bar.timestamp,
                tz=timezone(timedelta(hours=+8), "EST"),
            )
            - self.adjust(ktype),
            open=bar.openPrice,
            high=bar.highPrice,
            low=bar.lowPrice,
            close=bar.closePrice,
            volume=bar.volume,
        )

    def key(self, code, kline):
        return code + ":" + str(kline)
