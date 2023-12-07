from pathlib import Path
from typing import Any, Callable, Sequence
from desktop_notifier import Button, DesktopNotifier, Notification, ReplyField, Urgency


class Notify:
    _instance = DesktopNotifier()
    realtime = True

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
            await Notify._instance.send(
                title=title,
                message=message,
                urgency=urgency,
                sound=sound,
                thread=thread,
                attachment=attachment,
            )
