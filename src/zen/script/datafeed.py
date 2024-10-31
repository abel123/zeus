import asyncio
import os

from loguru import logger
from broker.enums import Resolution
from broker.mixed import Mixed


async def sync_to_local(lines):
    broker = Mixed()
    os.environ["ID"] = "233"
    for idx, l in enumerate(lines[1:]):
        l = l.split()
        logger.debug(f"{idx} {l}")
        coroutines = []
        for freq in [
            Resolution.Week,
            Resolution.Day,
            Resolution.Hour,
            Resolution.Min15,
        ]:
            coroutines.append(broker.subscribe(f":{l[0]}", freq, 0, 0, None, True))
        await asyncio.gather(*coroutines)
