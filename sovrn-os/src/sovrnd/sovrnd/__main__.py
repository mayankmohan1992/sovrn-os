"""Sovrn daemon entry point."""

import sys
import asyncio
import signal
from pathlib import Path

import uvicorn

from sovrnd.app import create_app
from sovrnd.config import load_config


def main():
    config_path = "/etc/sovrn/sovrnd.toml"
    args = sys.argv[1:]
    i = 0
    while i < len(args):
        if args[i] in ("--config", "-c") and i + 1 < len(args):
            config_path = args[i + 1]
            i += 2
        else:
            i += 1

    config = load_config(config_path)

    # Ensure directories exist
    Path(config.sockets_dir).mkdir(parents=True, exist_ok=True)
    Path(config.data_dir).mkdir(parents=True, exist_ok=True)

    app = create_app(config)

    uvicorn.run(
        app,
        host=config.host,
        port=config.port,
        uds=config.unix_socket,
        log_level=config.log_level.lower(),
        access_log=True,
    )


if __name__ == "__main__":
    main()
