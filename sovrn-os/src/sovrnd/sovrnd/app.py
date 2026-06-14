"""FastAPI application factory and routes."""

from pathlib import Path

from fastapi import FastAPI, HTTPException, Depends, WebSocket, WebSocketDisconnect
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel

from sovrnd.config import Config
from sovrnd.proxy import ServiceProxy
from sovrnd.auth import create_jwt, verify_jwt
from sovrnd.websocket import WebSocketManager


def create_app(config: Config) -> FastAPI:
    app = FastAPI(
        title="Sovrn OS Daemon",
        version="0.1.0",
        description="Central orchestrator API for Sovrn OS mesh services",
    )

    # CORS for PWA dev server
    app.add_middleware(
        CORSMiddleware,
        allow_origins=["http://localhost:54772", "http://127.0.0.1:54772"],
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )

    # Service proxy
    proxy = ServiceProxy(config.sockets_dir)

    # WebSocket manager
    ws_manager = WebSocketManager(proxy)

    # ── Health ────────────────────────────────────────────────
    @app.get("/api/health")
    async def health():
        services = [
            ("dht", "dht.health"),
            ("identity", "identity.get_profile"),
            ("presence", "presence.get_online"),
            ("feed", "feed.health"),
            ("mq", "mq.health"),
            ("cdn", "cdn.health"),
        ]
        results = await proxy.health_check_all(services)
        all_healthy = all(
            r.get("status") == "healthy" for r in results.values()
        )
        return {
            "status": "healthy" if all_healthy else "degraded",
            "services": results,
        }

    # ── Auth ──────────────────────────────────────────────────
    @app.post("/api/auth/create_identity")
    async def create_identity():
        result = await proxy.call("identity", "identity.import_identity", {
            "seed_phrase": "",
        })
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.post("/api/auth/login")
    async def login(body: dict):
        result = await proxy.call("identity", "identity.get_profile", {
            "public_key": body.get("public_key", ""),
        })
        if "error" in result:
            raise HTTPException(status_code=401, detail="Invalid credentials")
        profile = result.get("result", {})
        token = create_jwt(config, body.get("public_key", "unknown"))
        return {"token": token, "profile": profile}

    # ── DHT ───────────────────────────────────────────────────
    @app.get("/api/dht/lookup/{domain}")
    async def dht_lookup(domain: str):
        result = await proxy.call("dht", "dht.lookup", {"domain": domain})
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.post("/api/dht/register")
    async def dht_register(body: dict):
        result = await proxy.call("dht", "dht.register", body)
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.get("/api/dht/check/{name}")
    async def dht_check(name: str):
        result = await proxy.call("dht", "dht.check_available", {"name": name})
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    # ── Identity ──────────────────────────────────────────────
    @app.get("/api/identity/profile/{public_key}")
    async def get_profile(public_key: str):
        result = await proxy.call("identity", "identity.get_profile", {
            "public_key": public_key,
        })
        if "error" in result:
            raise HTTPException(status_code=504, detail=result["error"])
        return result.get("result", result)

    @app.put("/api/identity/profile")
    async def update_profile(body: dict):
        result = await proxy.call("identity", "identity.update_profile", body)
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.get("/api/identity/aliases/{key_id}")
    async def list_aliases(key_id: str):
        result = await proxy.call("identity", "identity.list_aliases", {"key_id": key_id})
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.post("/api/identity/alias")
    async def create_alias(body: dict):
        result = await proxy.call("identity", "identity.create_alias", body)
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.post("/api/identity/derive_app_key")
    async def derive_app_key(body: dict):
        result = await proxy.call("identity", "identity.derive_app_key", body)
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    # ── Presence ───────────────────────────────────────────────
    @app.get("/api/presence/online")
    async def presence_online(limit: int = 50):
        result = await proxy.call("presence", "presence.get_online", {"limit": limit})
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.post("/api/presence/status")
    async def presence_set_status(body: dict):
        result = await proxy.call("presence", "presence.set_status", body)
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    # ── Feed / Timeline ───────────────────────────────────────
    @app.get("/api/feed/timeline")
    async def feed_timeline(user_id: str = "default", limit: int = 50, cursor: str | None = None):
        params = {"user_id": user_id, "limit": limit}
        if cursor:
            params["cursor"] = cursor
        result = await proxy.call("feed", "feed.get_timeline", params)
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.get("/api/feed/event/{event_id}")
    async def feed_get_event(event_id: str):
        result = await proxy.call("feed", "feed.get_event", {"event_id": event_id})
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.post("/api/feed/create")
    async def feed_create_event(body: dict):
        result = await proxy.call("feed", "feed.create_event", body)
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.delete("/api/feed/event/{event_id}")
    async def feed_delete_event(event_id: str):
        result = await proxy.call("feed", "feed.delete_event", {"event_id": event_id})
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.post("/api/feed/react")
    async def feed_add_reaction(body: dict):
        result = await proxy.call("feed", "feed.add_reaction", body)
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.get("/api/feed/search")
    async def feed_search(q: str, kind: int | None = None, limit: int = 20):
        params = {"query": q, "limit": limit}
        if kind is not None:
            params["kind"] = kind
        result = await proxy.call("feed", "feed.search", params)
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    # ── Messages ──────────────────────────────────────────────
    @app.get("/api/messages/conversations")
    async def mq_conversations(user_id: str = "default", limit: int = 20):
        result = await proxy.call("mq", "mq.get_conversations", {
            "user_id": user_id, "limit": limit,
        })
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.get("/api/messages/{peer_id}")
    async def mq_messages(peer_id: str, user_id: str = "default", cursor: int | None = None, limit: int = 50):
        params = {"peer_id": peer_id, "user_id": user_id, "limit": limit}
        if cursor:
            params["cursor"] = cursor
        result = await proxy.call("mq", "mq.get_messages", params)
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.post("/api/messages/send")
    async def mq_send(body: dict):
        result = await proxy.call("mq", "mq.send_dm", body)
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.post("/api/messages/mark_read/{peer_id}")
    async def mq_mark_read(peer_id: str, body: dict):
        result = await proxy.call("mq", "mq.mark_read", {
            "peer_id": peer_id, "user_id": body.get("user_id", ""),
        })
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    # ── CDN ────────────────────────────────────────────────────
    @app.post("/api/cdn/push")
    async def cdn_push(body: dict):
        result = await proxy.call("cdn", "cdn.push", body)
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.get("/api/cdn/status/{push_id}")
    async def cdn_status(push_id: str):
        result = await proxy.call("cdn", "cdn.get_status", {"push_id": push_id})
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.get("/api/cdn/providers")
    async def cdn_providers():
        result = await proxy.call("cdn", "cdn.get_providers", {})
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    # ── WebSocket ──────────────────────────────────────────────
    @app.websocket("/ws")
    async def websocket_endpoint(websocket: WebSocket):
        await ws_manager.connect(websocket)
        try:
            while True:
                data = await websocket.receive_json()
                await ws_manager.handle_message(websocket, data)
        except WebSocketDisconnect:
            await ws_manager.disconnect(websocket)

    # ── Static files (PWA) ────────────────────────────────────
    pwa_dist = Path(config.data_dir) / "pwa-dist"
    if pwa_dist.exists():
        app.mount("/", StaticFiles(directory=str(pwa_dist), html=True), name="pwa")

    return app
