import logging
import sys
import datetime
from loguru import logger
from sanic import log


# 这是基于官方实现略微修改的版本
class InterceptHandler(logging.StreamHandler):
    def emit(self, record):
        # Get corresponding Loguru level if it exists
        try:
            level = logger.level(record.levelname).name
        except ValueError:
            level = record.levelno

        msg = self.format(
            record
        )  # 官方实现中使用record.getMessage()来获取msg，但在sanic中会漏掉它配置过的日志模板，因此要用self.format(record)
        logger.opt(depth=6, exception=record.exc_info).log(level, msg)


# 偷懒的实现版本
class SimpleInterceptHandler(logging.Handler):
    def emit(self, record):
        logger_opt = logger.opt(depth=6, exception=record.exc_info)
        msg = self.format(record)
        logger_opt.log(record.levelno, msg)


logging.basicConfig(handlers=[InterceptHandler()], level=logging.DEBUG, force=True)
logger.configure(handlers=[{"sink": sys.stdout, "level": "DEBUG"}])
logger.add("warning.log", level="WARNING")
logger.add("error.log", level="ERROR")
logging.getLogger("ib_insync.client").setLevel(logging.INFO)
