import asyncio
from typing import Any, Dict
from ib_insync import Contract, Event
from loguru import logger

from ib_insync.client import Connection
from moomoo.common.sys_config import SysConfig
from moomoo.quote.quote_query import InitConnect, KeepAlive
from moomoo.common import constant, utils

from moomoo_async.util import Periodic
import nest_asyncio

nest_asyncio.apply()


class Client:
    (DISCONNECTED, CONNECTING, CONNECTED) = range(3)

    def __init__(self, addr: str = "127.0.0.1", port: int = 9999):
        self.addr = addr
        self.port = port
        self.conn = Connection()
        self._logger = logger
        self.unique_id = 0
        SysConfig.set_proto_fmt(constant.ProtoFMT.Json)
        utils.get_unique_id32 = self._unique_id
        self.conn.hasData += self._onSocketHasData
        self.conn.disconnected += self._onSocketDisconnected
        self._config = None
        self.keep_alive_loop = Periodic(self.keep_alive, 0)

        self._reset()

    def _unique_id(self):
        self.unique_id += 1
        if self.unique_id >= 4294967295:
            self.unique_id = 1
        self._logger.debug("unique id {}", self.unique_id)

        return self.unique_id

    def _reset(self):
        self._data = b""
        self.connectState = Client.DISCONNECTED
        self._numBytesRecv = 0
        # futures and results are linked by key:
        self._futures: Dict[Any, asyncio.Future] = {}
        self._results: Dict[Any, Any] = {}
        self._reqId2Contract: Dict[int, Contract] = {}
        self._config = None
        self.subscribe_event = Event("subscribe")

    async def connect(self, timeout=2.0):
        timeout = timeout or None
        self.connectState = Client.CONNECTING
        await asyncio.wait_for(self.conn.connectAsync(self.addr, self.port), timeout)
        self._logger.info("Connected")
        await self._send_init()
        self.connectState = Client.CONNECTED

    def keep_alive(self):
        ret, msg, req = KeepAlive.pack_req(self.get_sync_conn_id())
        if ret != constant.RET_OK:
            logger.warning("KeepAlive.pack_req fail: {0}".format(msg))
            return
        self._logger.warning("send keep_alive")
        self.conn.sendMsg(req)

    async def _send_init(self):
        kargs = {
            "client_ver": int(SysConfig.get_client_ver()),
            "client_id": str(SysConfig.get_client_id()),
            "recv_notify": True,
            "is_encrypt": False,
            "push_proto_fmt": SysConfig.get_proto_fmt(),
        }

        ret, msg, req_str = InitConnect.pack_req(**kargs)
        if ret == constant.RET_OK:
            fut = self.startReq(self.unique_id)
            self.sendMsg(req_str)
            await fut
            rsp = fut.result()
            ret, msg, data = InitConnect.unpack_rsp(rsp)
            if ret != constant.RET_OK:
                return ret, msg

            self._logger.debug("init connect rsp {} {} {}", ret, msg, data)
            self._config = data
            self.keep_alive_loop.time = self._config["keep_alive_interval"]
            await self.keep_alive_loop.start()
            return constant.RET_OK, ""
        else:
            self._logger.error("Fail to pack InitConnect")
            return ret, msg

    def _onSocketHasData(self, data):
        self._data += data
        self._numBytesRecv += len(data)
        while len(self._data) > 0:
            head_len = utils.get_message_head_len()
            if len(self._data) < head_len:
                break
            head_dict = utils.parse_head(self._data[:head_len])
            body_len = head_dict["body_len"]
            if len(self._data) < head_len + body_len:
                break

            rsp_body = self._data[head_len : head_len + body_len]
            self._data = self._data[head_len + body_len :]
            rsp_pb = utils.binary2pb(
                rsp_body, head_dict["proto_id"], head_dict["proto_fmt_type"]
            )

            """self._logger.debug(
                "recv {} {} {} {}",
                head_dict["proto_id"],
                head_dict["serial_no"],
                rsp_pb,
                "",  # rsp_pb.retMsg,
            )"""
            if head_dict["proto_id"] == constant.ProtoId.Qot_UpdateKL:
                self.subscribe_event.emit(rsp_pb)
            else:
                self._endReq(head_dict["serial_no"], rsp_pb, True)

    def _onSocketDisconnected(self, msg):
        self._logger.error("disconnected", msg)
        asyncio.run(self.keep_alive_loop.stop())
        self._reset()

    def get_sync_conn_id(self):
        return self._config["conn_id"]

    def startReq(self, key, contract=None, container=None):
        """
        Start a new request and return the future that is associated
        with the key and container. The container is a list by default.
        """
        future: asyncio.Future = asyncio.Future()
        self._futures[key] = future
        self._results[key] = container if container is not None else []
        if contract:
            self._reqId2Contract[key] = contract
        return future

    def _endReq(self, key, result=None, success=True):
        """
        Finish the future of corresponding key with the given result.
        If no result is given then it will be popped of the general results.
        """
        future = self._futures.pop(key, None)
        self._reqId2Contract.pop(key, None)
        if future:
            if result is None:
                result = self._results.pop(key, [])
            if not future.done():
                if success:
                    future.set_result(result)
                else:
                    future.set_exception(result)

    def sendMsg(self, msg):
        if SysConfig.get_proto_fmt() == constant.ProtoFMT.Json:
            self._logger.debug("msg: {}", msg)
        self.conn.sendMsg(msg)
