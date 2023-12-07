from datetime import datetime, timedelta
from pathlib import Path
from typing import Any, Callable, Sequence
from cachetools import LRUCache, TTLCache
from desktop_notifier import Button, DesktopNotifier, Notification, ReplyField, Urgency
import pytz

class Notify:
    _instance = DesktopNotifier()
    realtime = True
    cache = TTLCache(maxsize=1000, ttl=timedelta(minutes=8).total_seconds())

    async def send(
        title: str,
        message: str,
        urgency: Urgency = Urgency.Normal,
        icon: Path | str | None = None,
        buttons: Sequence[Button] = (),
        reply_field: ReplyField | None = None,
        on_clicked: Callable[[], Any] | None = None,
        on_dismissed: Callable[[], Any] | None = None,
        attachment: Path | str | None = None,
        sound: bool = False,
        thread: str | None = None,
        timeout: int = -1,
    ) -> Notification:
        if True or Notify.realtime is True:
            cache = Notify.cache
            if cache.get((title, message, thread), None) == None:
                cache[(title, message, thread)] = True
                await Notify._instance.send(
                    title=f"{(datetime.now(pytz.timezone("Asia/Shanghai")).strftime("%m-%d %H:%M:%S"))}  {title}",
                    message=message,
                    urgency=urgency,
                    sound=sound,
                    thread=thread,
                    attachment=attachment,
                )
