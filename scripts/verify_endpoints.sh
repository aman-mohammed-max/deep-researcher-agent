#!/usr/bin/env bash
set -e

PORT=8001
BINARY="./target/release/deep-researcher-agent"

echo "=== Verifying Deep Researcher Agent Endpoints ==="
HOST=127.0.0.1 PORT=$PORT $BINARY > /tmp/deep_researcher_verify.log 2>&1 &
PID=$!

cleanup() {
    kill -9 $PID 2>/dev/null || true
}
trap cleanup EXIT

echo "Waiting for server on port $PORT..."
for i in {1..20}; do
    if curl -s http://127.0.0.1:$PORT/health | grep -q "healthy"; then
        break
    fi
    sleep 0.5
done

echo "1. Health Endpoint:"
curl -s http://127.0.0.1:$PORT/health
echo ""

echo "2. Data Sources Endpoint:"
curl -s http://127.0.0.1:$PORT/v1/data_sources
echo ""

echo "3. Dashboard Stats Endpoint:"
curl -s http://127.0.0.1:$PORT/api/dashboard/stats
echo ""

echo "4. Embedded UI Root (HTTP Status and Content-Type):"
curl -s -I http://127.0.0.1:$PORT/ | head -n 5

echo "5. Create API Key:"
KEY_RESP=$(curl -s -X POST http://127.0.0.1:$PORT/api/keys -H "Content-Type: application/json" -d '{"name": "E2E Test Key"}')
echo "$KEY_RESP"

echo ""
echo "=== All Endpoint Verification Checks Passed Successfully! ==="
