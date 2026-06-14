"""WebSocket connection manager for real-time push."""

import asyncio
import json
import logging
from fastapi import WebSocket

from sovrnd.proxy import ServiceProxy

logger = logging.getLogger(__name__)


class WebSocketManager:
    """Manages WebSocket connections and real-time event push."""

    def __init__(self, proxy: ServiceProxy):
        self.proxy = proxy
        self.connections: list[WebSocket] = []
        self._poll_task: asyncio.Task | None = None

    async def connect(self, websocket: WebSocket):
        await websocket.accept()
        self.connections.append(websocket)
        logger.info(f"WebSocket connected. Total: {len(self.connections)}")

    async def disconnect(self, websocket: WebSocket):
        self.connections.remove(websocket)
        logger.info(f"WebSocket disconnected. Total: {len(self.connections)}")

    async def broadcast(self, event_type: str, data: dict):
        """Broadcast an event to all connected clients."""
        message = json.dumps({"type": event_type, "data": data})
        dead = []
        for ws in self.connections:
            try:
                await ws.send_text(message)
            except Exception:
                dead.append(ws)
        for ws in dead:
            self.connections.remove(ws)

    async def handle_message(self, websocket: WebSocket, data: dict):
        """Handle incoming WebSocket message from client."""
        msg_type = data.get("type", "")

        if msg_type == "subscribe":
            # Client wants real-time updates
            await websocket.send_json({"type": "subscribed", "channels": data.get("channels", [])})

        elif msg_type == "presence.heartbeat":
            # Forward heartbeat to presence service
            result = await self.proxy.call("presence", "presence.heartbeat", data.get("params", {}))
            await websocket.send_json({"type": "presence.heartbeat_ack", "data": result.get("result", result)})

        elif msg_type == "feed.poll":
            # Poll for new feed events
            result = await self.proxy.call("feed", "feed.get_timeline", data.get("params", {}))
            await websocket.send_json({"type": "feed.update", "data": result.get("result", result)})

        elif msg_type == "messages.poll":
            # Poll for new messages
            result = await self.proxy.call("mq", "mq.get_conversations", data.get("params", {}))
            await websocket.send_json({"type": "messages.update", "data": result.get("result", result)})

        else:
            await websocket.send_json({"type": "error", "message": f"Unknown message type: {msg_type}"})
