"""Notification bridge core logic."""

import asyncio
import json
import logging
import subprocess
from dataclasses import dataclass
from enum import Enum

logger = logging.getLogger(__name__)


class NotificationType(Enum):
    MESSAGE = "message"
    FEED = "feed"
    PRESENCE = "presence"
    SYSTEM = "system"


@dataclass
class Notification:
    notif_type: NotificationType
    title: str
    body: str
    priority: str = "normal"  # low, normal, high, urgent
    icon: str | None = None
    category: str | None = None
    action_data: dict | None = None


class NotifyBridge:
    """Bridges Sovrn events to desktop notifications."""

    def __init__(self, sovrnd_ws_url: str = "ws://127.0.0.1:54771/ws"):
        self.sovrnd_ws_url = sovrnd_ws_url
        self._running = False

    async def start(self):
        """Connect to sovrnd WebSocket and start forwarding notifications."""
        self._running = True
        import websockets

        while self._running:
            try:
                async with websockets.connect(self.sovrnd_ws_url) as ws:
                    logger.info(f"Connected to sovrnd WebSocket at {self.sovrnd_ws_url}")
                    async for message in ws:
                        if not self._running:
                            break
                        try:
                            data = json.loads(message)
                            await self._handle_event(data)
                        except json.JSONDecodeError:
                            logger.warning(f"Invalid JSON from sovrnd: {message[:100]}")
            except Exception as e:
                logger.error(f"WebSocket connection error: {e}")
                if self._running:
                    await asyncio.sleep(5)  # Reconnect after 5s

    def stop(self):
        self._running = False

    async def _handle_event(self, data: dict):
        """Convert a sovrnd event to a desktop notification."""
        event_type = data.get("type", "")

        if event_type == "messages.update":
            notif = Notification(
                notif_type=NotificationType.MESSAGE,
                title="New Message",
                body=self._format_message(data.get("data", {})),
                category="im.received",
            )
        elif event_type == "feed.update":
            notif = Notification(
                notif_type=NotificationType.FEED,
                title="New Post",
                body=self._format_feed(data.get("data", {})),
                category="social.update",
            )
        elif event_type == "presence.heartbeat_ack":
            notif = Notification(
                notif_type=NotificationType.PRESENCE,
                title="Presence Update",
                body="Contact status changed",
                category="presence",
            )
        else:
            return  # Unknown event type, skip

        await self._send_notification(notif)

    def _format_message(self, data: dict) -> str:
        from_id = data.get("from_id", "Unknown")
        content = data.get("content", "")
        return f"From {from_id}: {content[:80]}"

    def _format_feed(self, data: dict) -> str:
        author = data.get("author", "Unknown")
        content = data.get("content", "")
        return f"{author}: {content[:80]}"

    async def _send_notification(self, notif: Notification):
        """Send notification via notify-send (libnotify)."""
        try:
            cmd = ["notify-send"]
            if notif.category:
                cmd.extend(["-c", notif.category])
            if notif.priority == "urgent":
                cmd.extend(["-u", "critical"])
            elif notif.priority == "high":
                cmd.extend(["-u", "normal"])
            else:
                cmd.extend(["-u", "low"])

            if notif.icon:
                cmd.extend(["-i", notif.icon])

            cmd.extend([notif.title, notif.body])

            process = await asyncio.create_subprocess_exec(
                *cmd,
                stdout=asyncio.subprocess.DEVNULL,
                stderr=asyncio.subprocess.DEVNULL,
            )
            await process.wait()
            logger.debug(f"Notification sent: {notif.title}")
        except FileNotFoundError:
            # notify-send not available (macOS, etc.)
            logger.debug(f"Notification (no notify-send): {notif.title}: {notif.body}")
        except Exception as e:
            logger.error(f"Failed to send notification: {e}")
