#!/usr/bin/env bash
# BDAI Render Keep-Alive Shell Script
URL="${1:-https://bdai-backend.onrender.com/api/health}"
INTERVAL="${2:-600}"

echo "=========================================================="
echo " Starting BDAI Backend Keep-Alive Heartbeat"
echo " Target:   $URL"
echo " Interval: ${INTERVAL}s"
echo "=========================================================="

while true; do
  DATE=$(date "+%Y-%m-%d %H:%M:%S")
  HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" --max-time 20 "$URL")
  if [ "$HTTP_CODE" = "200" ]; then
    echo "[$DATE] 🟢 [HTTP 200] Heartbeat successful for $URL"
  else
    echo "[$DATE] 🟡 [HTTP $HTTP_CODE] Ping returned non-200 status for $URL"
  fi
  sleep "$INTERVAL"
done
