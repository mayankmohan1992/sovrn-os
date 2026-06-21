"""Configuration loading for sovrnd."""

from dataclasses import dataclass, field
from pathlib import Path

try:
    import tomllib
except ImportError:
    import tomli as tomllib


@dataclass
class MeshServiceConfig:
    name: str
    socket_path: str
    health_method: str = "health"


@dataclass
class Config:
    host: str = "127.0.0.1"
    port: int = 54771
    unix_socket: str | None = None
    log_level: str = "info"
    sockets_dir: str = "/var/lib/sovrn/sockets"
    data_dir: str = "/var/lib/sovrn/sovrnd"
    jwt_secret: str = ""
    jwt_algorithm: str = "HS256"
    jwt_expiry_hours: int = 24

    mesh_services: list[MeshServiceConfig] = field(default_factory=lambda: [
        MeshServiceConfig(name="dht", socket_path="/var/lib/sovrn/sockets/dht.sock", health_method="dht.health"),
        MeshServiceConfig(name="identity", socket_path="/var/lib/sovrn/sockets/identity.sock", health_method="identity.health"),
        MeshServiceConfig(name="presence", socket_path="/var/lib/sovrn/sockets/presence.sock", health_method="presence.get_online"),
        MeshServiceConfig(name="feed", socket_path="/var/lib/sovrn/sockets/feed.sock", health_method="feed.health"),
        MeshServiceConfig(name="mq", socket_path="/var/lib/sovrn/sockets/mq.sock", health_method="mq.health"),
        MeshServiceConfig(name="cdn", socket_path="/var/lib/sovrn/sockets/cdn.sock", health_method="cdn.health"),
    ])


def load_config(path: str) -> Config:
    cfg = Config()
    try:
        data = tomllib.loads(Path(path).read_text())
        
        # Load server section
        server = data.get("server", {})
        cfg.host = server.get("host", data.get("host", cfg.host))
        cfg.port = server.get("port", data.get("port", cfg.port))
        cfg.unix_socket = server.get("unix_socket", data.get("unix_socket", cfg.unix_socket))
        cfg.log_level = server.get("log_level", data.get("log_level", cfg.log_level))
        
        # Load paths section
        paths = data.get("paths", {})
        cfg.sockets_dir = paths.get("sockets_dir", data.get("sockets_dir", cfg.sockets_dir))
        cfg.data_dir = paths.get("data_dir", data.get("data_dir", cfg.data_dir))
        
        # Load auth section
        auth = data.get("auth", {})
        cfg.jwt_secret = auth.get("jwt_secret", data.get("jwt_secret", cfg.jwt_secret))
        cfg.jwt_algorithm = auth.get("jwt_algorithm", data.get("jwt_algorithm", cfg.jwt_algorithm))
        cfg.jwt_expiry_hours = auth.get("jwt_expiry_hours", data.get("jwt_expiry_hours", cfg.jwt_expiry_hours))
        
        # Load mesh_services section if present
        if "mesh_services" in data:
            services_data = data["mesh_services"]
            mesh_services = []
            for name, svc_data in services_data.items():
                mesh_services.append(MeshServiceConfig(
                    name=svc_data.get("name", name),
                    socket_path=svc_data.get("socket_path", ""),
                    health_method=svc_data.get("health_method", "health"),
                ))
            if mesh_services:
                cfg.mesh_services = mesh_services
    except FileNotFoundError:
        pass  # Use defaults
    except Exception as e:
        print(f"Warning: Failed to load config {path}: {e}")

    # Generate JWT secret if not set
    if not cfg.jwt_secret:
        import secrets
        cfg.jwt_secret = secrets.token_urlsafe(32)

    return cfg
