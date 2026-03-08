#!/usr/bin/env bash
set -euo pipefail

PORT=3099
DB_PATH="./test_integration.db"
SERVER_PID=""
PASS=0
FAIL=0

cleanup() {
  if [ -n "$SERVER_PID" ]; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
  rm -f "$DB_PATH" "${DB_PATH}-wal" "${DB_PATH}-shm"
}
trap cleanup EXIT

# Build
echo "=== Building server ==="
nim c -d:release src/main.nim 2>&1 | tail -1

# Start server
DATABASE_PATH="$DB_PATH" PORT="$PORT" ./src/main &
SERVER_PID=$!

# Wait for server to be ready
echo "Waiting for server..."
for i in $(seq 1 20); do
  if curl -s -o /dev/null "http://localhost:${PORT}/" 2>/dev/null; then
    break
  fi
  if ! kill -0 "$SERVER_PID" 2>/dev/null; then
    echo "FATAL: Server failed to start"
    exit 1
  fi
  sleep 0.5
done

if ! kill -0 "$SERVER_PID" 2>/dev/null; then
  echo "FATAL: Server failed to start"
  exit 1
fi

BASE="http://localhost:${PORT}"

assert_status() {
  local label="$1" method="$2" url="$3" expected="$4"
  shift 4
  local status
  status=$(curl -s -o /dev/null -w '%{http_code}' -X "$method" "$@" "$url")
  if [ "$status" = "$expected" ]; then
    echo "  PASS  $label (HTTP $status)"
    PASS=$((PASS + 1))
  else
    echo "  FAIL  $label (expected $expected, got $status)"
    FAIL=$((FAIL + 1))
  fi
}

assert_json_contains() {
  local label="$1" method="$2" url="$3" pattern="$4"
  shift 4
  local body
  body=$(curl -s -X "$method" "$@" "$url")
  if echo "$body" | grep -q "$pattern"; then
    echo "  PASS  $label"
    PASS=$((PASS + 1))
  else
    echo "  FAIL  $label (pattern '$pattern' not found in response)"
    echo "        body: ${body:0:200}"
    FAIL=$((FAIL + 1))
  fi
}

assert_json_eq() {
  local label="$1" method="$2" url="$3" expected="$4"
  shift 4
  local body
  body=$(curl -s -X "$method" "$@" "$url")
  if [ "$body" = "$expected" ]; then
    echo "  PASS  $label"
    PASS=$((PASS + 1))
  else
    echo "  FAIL  $label (expected '$expected', got '$body')"
    FAIL=$((FAIL + 1))
  fi
}

echo ""
echo "=== SPA ==="
assert_status "GET / returns 200" GET "$BASE/" 200
assert_json_contains "GET / returns HTML" GET "$BASE/" "<!doctype html>"
assert_json_contains "GET / injects BASE_PATH" GET "$BASE/" 'window.__BASE_PATH__'

echo ""
echo "=== Static files ==="
assert_status "GET /favicon.svg returns 200" GET "$BASE/favicon.svg" 200

echo ""
echo "=== Favorites (empty) ==="
assert_json_eq "GET /api/favorites returns empty array" GET "$BASE/api/favorites" "[]"

echo ""
echo "=== Favorites CRUD ==="
assert_status "PUT /api/favorites creates entry" PUT "$BASE/api/favorites/narou/n1234ab" 200 \
  -H "Content-Type: application/json" -d '{"title":"Test Novel","page":10}'
assert_json_contains "GET /api/favorites returns created entry" GET "$BASE/api/favorites" '"id":"n1234ab"'
assert_json_contains "PATCH progress updates read position" PATCH "$BASE/api/favorites/narou/n1234ab/progress" '"read":5' \
  -H "Content-Type: application/json" -d '{"read":5}'
assert_json_contains "DELETE removes entry" DELETE "$BASE/api/favorites/narou/n1234ab" '"ok":true'
assert_json_eq "GET /api/favorites is empty after delete" GET "$BASE/api/favorites" "[]"

echo ""
echo "=== Error handling ==="
assert_status "Invalid type returns 400" GET "$BASE/api/novel/invalid/ranking" 400
assert_json_contains "Invalid type error message" GET "$BASE/api/novel/invalid/ranking" '"error":"Invalid type"'
assert_status "Missing q param returns 400" GET "$BASE/api/novel/narou/search" 400
assert_status "PUT without title returns 400" PUT "$BASE/api/favorites/narou/test" 400 \
  -H "Content-Type: application/json" -d '{}'
assert_status "PATCH progress without read returns 400" PATCH "$BASE/api/favorites/narou/test/progress" 400 \
  -H "Content-Type: application/json" -d '{}'
assert_status "Quarter ranking for kakuyomu returns 400" GET "$BASE/api/novel/kakuyomu/ranking?period=quarter" 400
assert_status "Invalid period returns 400" GET "$BASE/api/novel/narou/ranking?period=invalid" 400

if [ "${LIVE_TESTS:-0}" = "1" ]; then
  echo ""
  echo "=== Ranking (live) ==="
  assert_status "GET narou ranking returns 200" GET "$BASE/api/novel/narou/ranking?period=daily" 200
  assert_json_contains "Narou ranking has genre key" GET "$BASE/api/novel/narou/ranking?period=daily" '"異世界 \[恋愛\]"'
  assert_json_contains "Narou ranking items have id field" GET "$BASE/api/novel/narou/ranking?period=daily" '"id":'
  assert_status "PATCH ranking (cache bypass) returns 200" PATCH "$BASE/api/novel/narou/ranking?period=daily" 200
fi

echo ""
echo "=== PATCH progress for non-existent favorite ==="
assert_json_contains "PATCH progress for missing favorite returns ok" PATCH "$BASE/api/favorites/narou/nonexistent/progress" '"ok":true' \
  -H "Content-Type: application/json" -d '{"read":1}'

echo ""
echo "===================="
echo "Results: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] && echo "All tests passed!" || exit 1
