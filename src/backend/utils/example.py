import asyncio
from wechat_push import WechatMessagePush

pushed = WechatMessagePush(
    "wx0be6c71a20ed9d6b",
    "fd2be5de2589836d60f3c5cd15663589",
    "r3K--tW8lKnPW8CC4E977EDv6PLOyOZRBNIeiDEvR-o",
)
task = pushed.send_wechat_temple_msg("haha test")
asyncio.run(task)
