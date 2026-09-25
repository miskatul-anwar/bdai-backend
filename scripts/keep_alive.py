#!/usr/bin/env python3
"""
BDAI Backend Keep-Alive Daemon
==============================
Prevents free-tier Render instances from sleeping by sending periodic HTTP GET
heartbeats to the backend's /api/health endpoint.

Usage:
  python3 scripts/keep_alive.py
  python3 scripts/keep_alive.py --interval 300
  python3 scripts/keep_alive.py --url https://bdai-backend.onrender.com/api/health
  python3 scripts/keep_alive.py --once
"""

import sys
import time
import argparse
import json
import signal
from datetime import datetime
from urllib.request import Request, urlopen
from urllib.error import URLError, HTTPError

DEFAULT_URL = "https://bdai-backend.onrender.com/api/health"
DEFAULT_INTERVAL_SECS = 600  # 10 minutes (Render free-tier sleeps after 15 min of inactivity)
USER_AGENT = "BDAI-KeepAlive-Heartbeat/1.0"

def log(msg: str):
    now = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    print(f"[{now}] {msg}", flush=True)

def ping_once(url: str, timeout: int = 25) -> bool:
    req = Request(url, headers={"User-Agent": USER_AGENT, "Accept": "application/json"})
    start_time = time.time()
    try:
        with urlopen(req, timeout=timeout) as response:
            latency_ms = int((time.time() - start_time) * 1000)
            status = response.getcode()
            body = response.read().decode("utf-8", errors="replace")
            try:
                parsed = json.loads(body)
                service_status = parsed.get("status", "unknown")
                db_status = parsed.get("database", "unknown")
                log(f"🟢 [HTTP {status}] {url} ({latency_ms}ms) | Service: {service_status} | DB: {db_status}")
            except Exception:
                log(f"🟢 [HTTP {status}] {url} ({latency_ms}ms)")
            return True
    except HTTPError as e:
        latency_ms = int((time.time() - start_time) * 1000)
        log(f"🟡 [HTTP {e.code}] {url} ({latency_ms}ms) - {e.reason}")
        return False
    except URLError as e:
        latency_ms = int((time.time() - start_time) * 1000)
        log(f"🔴 [Error] {url} ({latency_ms}ms) - Network unreachable: {e.reason}")
        return False
    except Exception as e:
        latency_ms = int((time.time() - start_time) * 1000)
        log(f"🔴 [Error] {url} ({latency_ms}ms) - {e}")
        return False

def main():
    parser = argparse.ArgumentParser(description="BDAI Backend Keep-Alive Daemon")
    parser.add_argument("--url", default=DEFAULT_URL, help=f"Health endpoint URL (default: {DEFAULT_URL})")
    parser.add_argument("--interval", type=int, default=DEFAULT_INTERVAL_SECS, help=f"Interval in seconds between pings (default: {DEFAULT_INTERVAL_SECS}s)")
    parser.add_argument("--once", action="store_true", help="Run a single ping check and exit immediately")
    args = parser.parse_args()

    def handle_exit(signum, frame):
        log("Keep-Alive daemon stopping gracefully...")
        sys.exit(0)

    signal.signal(signal.SIGINT, handle_exit)
    signal.signal(signal.SIGTERM, handle_exit)

    print("=" * 65)
    print("        BDAI Render Free-Tier Keep-Alive Daemon Active          ")
    print(f"  Target URL: {args.url}")
    print(f"  Interval:   {args.interval}s ({args.interval // 60}m {args.interval % 60}s)")
    print("=" * 65, flush=True)

    if args.once:
        success = ping_once(args.url)
        sys.exit(0 if success else 1)

    while True:
        ping_once(args.url)
        time.sleep(args.interval)

if __name__ == "__main__":
    main()
