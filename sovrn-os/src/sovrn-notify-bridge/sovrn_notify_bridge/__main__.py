"""Notify bridge entry point."""

import asyncio
import logging
import uvicorn

from sovrn_notify_bridge.bridge import NotifyBridge

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


def main():
    """Start the notify bridge service.

    Runs both:
    1. FastAPI for direct notification requests (port 54773)
    2. WebSocket client subscribing to sovrnd events
    """
    bridge = NotifyBridge()

    # Start WebSocket bridge in background
    async def run_bridge_and_api():
        # Start the bridge in a task
        bridge_task = asyncio.create_task(bridge.start())

        # Run FastAPI
        config = uvicorn.Config(
            "sovrn_notify_bridge.app:app",
            host="127.0.0.1",
            port=54773,
            log_level="info",
        )
        server = uvicorn.Server(config)

        try:
            await server.serve()
        finally:
            bridge.stop()
            bridge_task.cancel()

    asyncio.run(run_bridge_and_api())


if __name__ == "__main__":
    main()
