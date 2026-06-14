"""Sovrn Notify Bridge — Bridges push notifications from sovrnd to OS notification daemon.

Subscribes to WebSocket events from sovrnd and forwards them to
the desktop notification system (libnotify / org.freedesktop.Notifications).
Runs on port 54773 as documented in the architecture.
"""

__version__ = "0.1.0"
