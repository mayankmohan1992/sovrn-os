"""Notify bridge FastAPI application."""

from fastapi import FastAPI
from pydantic import BaseModel

from sovrn_notify_bridge.bridge import NotifyBridge, Notification, NotificationType

app = FastAPI(title="Sovrn Notify Bridge", version="0.1.0")

# Global bridge instance
_bridge: NotifyBridge | None = None


class NotificationRequest(BaseModel):
    title: str
    body: str
    priority: str = "normal"
    category: str | None = None
    icon: str | None = None


@app.get("/health")
async def health():
    return {"status": "ok", "service": "sovrn-notify-bridge"}


@app.post("/notify")
async def notify(body: NotificationRequest):
    """Send a notification directly."""
    notif = Notification(
        notif_type=NotificationType.SYSTEM,
        title=body.title,
        body=body.body,
        priority=body.priority,
        category=body.category,
        icon=body.icon,
    )
    bridge = NotifyBridge()
    await bridge._send_notification(notif)
    return {"sent": True, "title": body.title}
