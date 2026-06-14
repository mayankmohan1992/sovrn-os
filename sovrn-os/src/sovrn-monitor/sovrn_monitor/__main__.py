"""Monitor service entry point."""

import uvicorn


def main():
    uvicorn.run("sovrn_monitor.app:app", host="127.0.0.1", port=54777, log_level="info")


if __name__ == "__main__":
    main()
