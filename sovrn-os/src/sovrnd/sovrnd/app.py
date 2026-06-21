"""FastAPI application factory and routes."""

from pathlib import Path

from fastapi import FastAPI, HTTPException, Depends, WebSocket, WebSocketDisconnect, Header
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

    # JWT extraction dependency
    async def get_current_user(authorization: str | None = Header(None)) -> str:
        if not authorization:
            raise HTTPException(status_code=401, detail="Missing authorization header")
        if not authorization.startswith("Bearer "):
            raise HTTPException(status_code=401, detail="Invalid token format")
        token = authorization.split(" ")[1]
        token_data = verify_jwt(config, token)
        if not token_data:
            raise HTTPException(status_code=401, detail="Invalid token")
        return token_data.public_key

    def resolve_key(user_key: str, current_user: str) -> str:
        if user_key == "me" or user_key == "default":
            return current_user
        return user_key

    def resolve_did(key_id: str, current_user: str) -> str:
        if key_id == "me" or key_id == "default":
            return f"did:mesh:{current_user}"
        return key_id

    # ── Health ────────────────────────────────────────────────
    @app.get("/api/health")
    async def health():
        services = [
            ("dht", "dht.health"),
            ("identity", "identity.health"),
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
        val = body.get("public_key", "").strip()
        if not val:
            raise HTTPException(status_code=400, detail="Missing public_key or seed phrase")
        
        if val == "new":
            result = await proxy.call("identity", "identity.import_identity", {
                "seed_phrase": "",
            })
        elif len(val) > 16 or " " in val:
            result = await proxy.call("identity", "identity.import_identity", {
                "seed_phrase": val,
            })
        else:
            result = await proxy.call("identity", "identity.get_profile", {
                "public_key": val,
            })
            
        if "error" in result:
            raise HTTPException(status_code=401, detail=result["error"])
            
        profile = result.get("result", {})
        pubkey_hash = profile.get("public_key", val)
        token = create_jwt(config, pubkey_hash)
        return {"token": token, "profile": profile}

    # ── DHT ───────────────────────────────────────────────────
    @app.get("/api/dht/lookup/{domain}")
    async def dht_lookup(domain: str, current_user: str = Depends(get_current_user)):
        result = await proxy.call("dht", "dht.lookup", {"domain": domain})
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.post("/api/dht/register")
    async def dht_register(body: dict, current_user: str = Depends(get_current_user)):
        result = await proxy.call("dht", "dht.register", body)
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.get("/api/dht/check/{name}")
    async def dht_check(name: str, current_user: str = Depends(get_current_user)):
        result = await proxy.call("dht", "dht.check_available", {"name": name})
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    # ── Identity ──────────────────────────────────────────────
    @app.get("/api/identity/profile/{public_key}")
    async def get_profile(public_key: str, current_user: str = Depends(get_current_user)):
        pk = resolve_key(public_key, current_user)
        result = await proxy.call("identity", "identity.get_profile", {
            "public_key": pk,
        })
        if "error" in result:
            raise HTTPException(status_code=504, detail=result["error"])
        return result.get("result", result)

    @app.put("/api/identity/profile")
    async def update_profile(body: dict, current_user: str = Depends(get_current_user)):
        if "public_key" not in body or body["public_key"] == "me":
            body["public_key"] = current_user
        result = await proxy.call("identity", "identity.update_profile", body)
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.get("/api/identity/aliases/{key_id}")
    async def list_aliases(key_id: str, current_user: str = Depends(get_current_user)):
        resolved_kid = resolve_did(key_id, current_user)
        result = await proxy.call("identity", "identity.list_aliases", {"key_id": resolved_kid})
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.post("/api/identity/alias")
    async def create_alias(body: dict, current_user: str = Depends(get_current_user)):
        key_id = body.get("key_id", "me")
        resolved_kid = resolve_did(key_id, current_user)
        result = await proxy.call("identity", "identity.create_alias", {
            "key_id": resolved_kid,
            "name": body.get("name", "")
        })
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.post("/api/identity/derive_app_key")
    async def derive_app_key(body: dict, current_user: str = Depends(get_current_user)):
        result = await proxy.call("identity", "identity.derive_app_key", body)
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.get("/api/identity/directory/users")
    async def get_directory_users(current_user: str = Depends(get_current_user)):
        result = await proxy.call("identity", "identity.list_all_identities", {})
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.get("/api/identity/directory/domains")
    async def get_directory_domains(current_user: str = Depends(get_current_user)):
        result = await proxy.call("identity", "identity.list_all_domains", {})
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    # ── Presence ───────────────────────────────────────────────
    @app.get("/api/presence/online")
    async def presence_online(limit: int = 50, current_user: str = Depends(get_current_user)):
        result = await proxy.call("presence", "presence.get_online", {"limit": limit})
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.post("/api/presence/status")
    async def presence_set_status(body: dict, current_user: str = Depends(get_current_user)):
        result = await proxy.call("presence", "presence.set_status", body)
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    # ── Feed / Timeline ───────────────────────────────────────
    @app.get("/api/feed/timeline")
    async def feed_timeline(user_id: str = "default", limit: int = 50, cursor: str | None = None, current_user: str = Depends(get_current_user)):
        uid = resolve_key(user_id, current_user)
        params = {"user_id": uid, "limit": limit}
        if cursor:
            params["cursor"] = cursor
        result = await proxy.call("feed", "feed.get_timeline", params)
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.get("/api/feed/event/{event_id}")
    async def feed_get_event(event_id: str, current_user: str = Depends(get_current_user)):
        result = await proxy.call("feed", "feed.get_event", {"event_id": event_id})
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.post("/api/feed/create")
    async def feed_create_event(body: dict, current_user: str = Depends(get_current_user)):
        if body.get("author") == "me":
            body["author"] = current_user
        result = await proxy.call("feed", "feed.create_event", body)
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.delete("/api/feed/event/{event_id}")
    async def feed_delete_event(event_id: str, current_user: str = Depends(get_current_user)):
        result = await proxy.call("feed", "feed.delete_event", {"event_id": event_id})
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.post("/api/feed/react")
    async def feed_add_reaction(body: dict, current_user: str = Depends(get_current_user)):
        result = await proxy.call("feed", "feed.add_reaction", body)
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.get("/api/feed/search")
    async def feed_search(q: str, kind: int | None = None, limit: int = 20, current_user: str = Depends(get_current_user)):
        params = {"query": q, "limit": limit}
        if kind is not None:
            params["kind"] = kind
        result = await proxy.call("feed", "feed.search", params)
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    # ── Messages ──────────────────────────────────────────────
    @app.get("/api/messages/conversations")
    async def mq_conversations(user_id: str = "default", limit: int = 20, current_user: str = Depends(get_current_user)):
        uid = resolve_key(user_id, current_user)
        result = await proxy.call("mq", "mq.get_conversations", {
            "user_id": uid, "limit": limit,
        })
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.get("/api/messages/{peer_id}")
    async def mq_messages(peer_id: str, user_id: str = "default", cursor: int | None = None, limit: int = 50, current_user: str = Depends(get_current_user)):
        uid = resolve_key(user_id, current_user)
        params = {"peer_id": peer_id, "user_id": uid, "limit": limit}
        if cursor:
            params["cursor"] = cursor
        result = await proxy.call("mq", "mq.get_messages", params)
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.post("/api/messages/send")
    async def mq_send(body: dict, current_user: str = Depends(get_current_user)):
        if "from_id" not in body or body["from_id"] == "me":
            body["from_id"] = current_user
        result = await proxy.call("mq", "mq.send_dm", body)
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.post("/api/messages/mark_read/{peer_id}")
    async def mq_mark_read(peer_id: str, body: dict, current_user: str = Depends(get_current_user)):
        uid = body.get("user_id")
        if not uid or uid == "me" or uid == "default":
            uid = current_user
        result = await proxy.call("mq", "mq.mark_read", {
            "peer_id": peer_id, "user_id": uid,
        })
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    # ── CDN ────────────────────────────────────────────────────
    @app.post("/api/cdn/push")
    async def cdn_push(body: dict, current_user: str = Depends(get_current_user)):
        result = await proxy.call("cdn", "cdn.push", body)
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.get("/api/cdn/status/{push_id}")
    async def cdn_status(push_id: str, current_user: str = Depends(get_current_user)):
        result = await proxy.call("cdn", "cdn.get_status", {"push_id": push_id})
        if "error" in result:
            raise HTTPException(status_code=502, detail=result["error"])
        return result.get("result", result)

    @app.get("/api/cdn/providers")
    async def cdn_providers(current_user: str = Depends(get_current_user)):
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
