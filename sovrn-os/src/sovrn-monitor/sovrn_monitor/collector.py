"""System metrics collector using psutil."""

import psutil
from dataclasses import dataclass, field
from typing import Optional


@dataclass
class SystemMetrics:
    cpu_percent: float
    cpu_count: int
    memory_total: int
    memory_used: int
    memory_percent: float
    disk_total: int
    disk_used: int
    disk_percent: float
    network_sent: int
    network_recv: int
    uptime_seconds: float
    load_avg: tuple[float, float, float]


@dataclass
class ServiceHealth:
    name: str
    status: str
    pid: Optional[int] = None
    cpu_percent: float = 0.0
    memory_mb: float = 0.0
    uptime_seconds: float = 0.0


_boot_time: Optional[float] = None


def collect_system_metrics() -> SystemMetrics:
    """Collect current system metrics."""
    global _boot_time
    if _boot_time is None:
        _boot_time = psutil.boot_time()

    import time
    uptime = time.time() - _boot_time

    mem = psutil.virtual_memory()
    disk = psutil.disk_usage("/")
    net = psutil.net_io_counters()

    return SystemMetrics(
        cpu_percent=psutil.cpu_percent(interval=0.1),
        cpu_count=psutil.cpu_count(),
        memory_total=mem.total,
        memory_used=mem.used,
        memory_percent=mem.percent,
        disk_total=disk.total,
        disk_used=disk.used,
        disk_percent=disk.percent,
        network_sent=net.bytes_sent,
        network_recv=net.bytes_recv,
        uptime_seconds=uptime,
        load_avg=psutil.getloadavg() if hasattr(psutil, "getloadavg") else (0, 0, 0),
    )


def check_service_health(service_name: str) -> ServiceHealth:
    """Check health of a Sovrn service by name."""
    status = "unknown"
    pid = None
    cpu_pct = 0.0
    mem_mb = 0.0
    uptime = 0.0

    # Try to find the service process
    for proc in psutil.process_iter(["pid", "name", "cmdline", "cpu_percent", "memory_info", "create_time"]):
        try:
            cmdline = " ".join(proc.info.get("cmdline") or [])
            if service_name in cmdline or service_name in (proc.info.get("name") or ""):
                pid = proc.info["pid"]
                cpu_pct = proc.info.get("cpu_percent") or 0.0
                mem_info = proc.info.get("memory_info")
                mem_mb = mem_info.rss / 1024 / 1024 if mem_info else 0.0
                create_time = proc.info.get("create_time")
                if create_time:
                    import time
                    uptime = time.time() - create_time
                status = "running"
                break
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            continue

    if status != "running":
        status = "stopped"

    return ServiceHealth(
        name=service_name,
        status=status,
        pid=pid,
        cpu_percent=cpu_pct,
        memory_mb=mem_mb,
        uptime_seconds=uptime,
    )


# Alert thresholds
ALERT_THRESHOLDS = {
    "cpu_percent": 90.0,
    "memory_percent": 85.0,
    "disk_percent": 90.0,
}


def check_alerts(metrics: SystemMetrics) -> list[dict]:
    """Check if any metrics exceed alert thresholds."""
    alerts = []
    if metrics.cpu_percent > ALERT_THRESHOLDS["cpu_percent"]:
        alerts.append({
            "level": "warning",
            "metric": "cpu",
            "value": metrics.cpu_percent,
            "threshold": ALERT_THRESHOLDS["cpu_percent"],
            "message": f"CPU usage {metrics.cpu_percent:.1f}% exceeds threshold",
        })
    if metrics.memory_percent > ALERT_THRESHOLDS["memory_percent"]:
        alerts.append({
            "level": "warning",
            "metric": "memory",
            "value": metrics.memory_percent,
            "threshold": ALERT_THRESHOLDS["memory_percent"],
            "message": f"Memory usage {metrics.memory_percent:.1f}% exceeds threshold",
        })
    if metrics.disk_percent > ALERT_THRESHOLDS["disk_percent"]:
        alerts.append({
            "level": "critical",
            "metric": "disk",
            "value": metrics.disk_percent,
            "threshold": ALERT_THRESHOLDS["disk_percent"],
            "message": f"Disk usage {metrics.disk_percent:.1f}% exceeds threshold",
        })
    return alerts
