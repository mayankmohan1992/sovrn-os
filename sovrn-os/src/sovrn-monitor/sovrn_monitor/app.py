"""Monitor service FastAPI application."""

from fastapi import FastAPI
from sovrn_monitor.collector import collect_system_metrics, check_service_health, check_alerts

app = FastAPI(title="Sovrn Monitor Service", version="0.1.0")

SOVRN_SERVICES = [
    "sovrnd",
    "sovrn-dht",
    "sovrn-identity",
    "sovrn-presence",
    "sovrn-feed",
    "sovrn-message-queue",
    "sovrn-cdn-agent",
    "yggdrasil",
    "caddy",
    "NetworkManager",
]


@app.get("/health")
async def health():
    return {"status": "ok", "service": "sovrn-monitor"}


@app.get("/metrics")
async def metrics():
    sys_metrics = collect_system_metrics()
    return {
        "cpu": {
            "percent": sys_metrics.cpu_percent,
            "count": sys_metrics.cpu_count,
            "load_avg": list(sys_metrics.load_avg),
        },
        "memory": {
            "total": sys_metrics.memory_total,
            "used": sys_metrics.memory_used,
            "percent": sys_metrics.memory_percent,
        },
        "disk": {
            "total": sys_metrics.disk_total,
            "used": sys_metrics.disk_used,
            "percent": sys_metrics.disk_percent,
        },
        "network": {
            "bytes_sent": sys_metrics.network_sent,
            "bytes_recv": sys_metrics.network_recv,
        },
        "uptime": sys_metrics.uptime_seconds,
    }


@app.get("/services")
async def services():
    results = {}
    for svc in SOVRN_SERVICES:
        health = check_service_health(svc)
        results[svc] = {
            "status": health.status,
            "pid": health.pid,
            "cpu_percent": health.cpu_percent,
            "memory_mb": round(health.memory_mb, 1),
            "uptime_seconds": round(health.uptime_seconds, 1),
        }
    return {"services": results}


@app.get("/alerts")
async def alerts():
    sys_metrics = collect_system_metrics()
    alert_list = check_alerts(sys_metrics)
    return {"alerts": alert_list}
