"""Unix socket proxy for mesh service communication.

Sends JSON-RPC 2.0 requests to mesh services over Unix domain sockets
and returns the responses.
"""

import asyncio
import json
import logging
from pathlib import Path

logger = logging.getLogger(__name__)


class UnixSocketProxy:
    """Async JSON-RPC 2.0 client over Unix domain sockets."""

    def __init__(self, socket_path: str, timeout: float = 5.0):
        self.socket_path = socket_path
        self.timeout = timeout

    async def call(self, method: str, params: dict | None = None, request_id: int = 1) -> dict:
        """Send a JSON-RPC 2.0 request and return the response."""
        if params is None:
            params = {}

        request = {
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": request_id,
        }

        try:
            reader, writer = await asyncio.wait_for(
                asyncio.open_unix_connection(self.socket_path),
                timeout=self.timeout,
            )
        except (ConnectionRefusedError, FileNotFoundError, asyncio.TimeoutError) as e:
            logger.warning(f"Failed to connect to {self.socket_path}: {e}")
            return {"error": {"code": -32000, "message": f"Service unavailable: {e}"}}

        try:
            data = json.dumps(request).encode() + b"\n"
            writer.write(data)
            await writer.drain()

            response_data = await asyncio.wait_for(reader.readline(), timeout=self.timeout)
            if not response_data:
                return {"error": {"code": -32001, "message": "Empty response from service"}}

            response = json.loads(response_data)
            return response

        except (asyncio.TimeoutError, json.JSONDecodeError, ConnectionError) as e:
            logger.error(f"Error communicating with {self.socket_path}: {e}")
            return {"error": {"code": -32002, "message": f"Communication error: {e}"}}
        finally:
            writer.close()
            await writer.wait_closed()

    async def health_check(self, method: str = "health") -> dict:
        """Check if the service is responsive."""
        response = await self.call(method)
        if "error" in response:
            return {"status": "unhealthy", "error": response.get("error", {})}
        return {"status": "healthy", "result": response.get("result", {})}


class ServiceProxy:
    """High-level proxy to all mesh services."""

    def __init__(self, sockets_dir: str = "/var/lib/sovrn/sockets"):
        self.sockets_dir = sockets_dir
        self._proxies: dict[str, UnixSocketProxy] = {}

    def get_proxy(self, service: str) -> UnixSocketProxy:
        if service not in self._proxies:
            socket_path = str(Path(self.sockets_dir) / f"{service}.sock")
            self._proxies[service] = UnixSocketProxy(socket_path)
        return self._proxies[service]

    async def call(self, service: str, method: str, params: dict | None = None) -> dict:
        proxy = self.get_proxy(service)
        return await proxy.call(method, params)

    async def health_check(self, service: str, method: str = "health") -> dict:
        proxy = self.get_proxy(service)
        return await proxy.health_check(method)

    async def health_check_all(self, services: list[tuple[str, str]]) -> dict:
        results = {}
        tasks = []
        for service, method in services:
            tasks.append(self.health_check(service, method))
        responses = await asyncio.gather(*tasks, return_exceptions=True)
        for (service, _), response in zip(services, responses):
            if isinstance(response, Exception):
                results[service] = {"status": "error", "error": str(response)}
            else:
                results[service] = response
        return results
