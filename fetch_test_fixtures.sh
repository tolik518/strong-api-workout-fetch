#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# fetch_test_fixtures.sh
#
# Fetches real API responses from the Strong backend and saves them as
# fixture files in strong-api-lib/tests/fixtures/.
#
# Usage:
#   STRONG_USER=you@example.com STRONG_PASS=yourpassword ./fetch_test_fixtures.sh
#
# Optionally set BASE_URL if you are pointing at a different backend:
#   BASE_URL=https://api.strong.app ./fetch_test_fixtures.sh
# ---------------------------------------------------------------------------

set -uo pipefail

# Load .env from the project root if it exists (so STRONG_USER, STRONG_PASS,
# and STRONG_BACKEND don't have to be passed manually every time).
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
if [[ -f "${SCRIPT_DIR}/.env" ]]; then
  echo "==> Loading ${SCRIPT_DIR}/.env"
  set -a
  # shellcheck source=/dev/null
  source "${SCRIPT_DIR}/.env"
  set +a
fi

BASE_URL="${BASE_URL:-${STRONG_BACKEND:-https://back.strong.app}}"
FIXTURES_DIR="$(dirname "$0")/strong-api-lib/tests/fixtures"

: "${STRONG_USER:?Set STRONG_USER to your Strong username or e-mail}"
: "${STRONG_PASS:?Set STRONG_PASS to your Strong password}"

echo "==> Logging in..."
LOGIN_JSON=$(curl -s --max-time 15 --connect-timeout 10 -X POST "${BASE_URL}/auth/login" \
  -H "User-Agent: Strong Android" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -H "x-client-build: 600013" \
  -H "x-client-platform: android" \
  -d "{\"usernameOrEmail\": \"${STRONG_USER}\", \"password\": \"${STRONG_PASS}\"}" \
  || { echo "ERROR: curl request failed (timeout or network issue? Check BASE_URL=${BASE_URL})"; exit 1; })

echo "    Raw login response: $LOGIN_JSON"

ACCESS_TOKEN=$(echo "$LOGIN_JSON" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
except json.JSONDecodeError as e:
    print(f'ERROR: response is not valid JSON: {e}', file=sys.stderr)
    sys.exit(1)
token = data.get('accessToken')
if not token:
    print(f'ERROR: no accessToken in response. Keys present: {list(data.keys())}', file=sys.stderr)
    sys.exit(1)
print(token)
") || exit 1

USER_ID=$(echo "$LOGIN_JSON" | python3 -c "
import sys, json
data = json.load(sys.stdin)
uid = data.get('userId')
if not uid:
    print(f'ERROR: no userId in response. Keys present: {list(data.keys())}', file=sys.stderr)
    sys.exit(1)
print(uid)
") || exit 1

echo "$LOGIN_JSON" | python3 -m json.tool > "${FIXTURES_DIR}/login_response.json"
echo "    Saved login_response.json"

echo "    Logged in as user ${USER_ID}"

# ---------------------------------------------------------------------------
# GET /api/measurements  (page 1)
# ---------------------------------------------------------------------------
echo "==> Fetching measurements (page 1)..."
curl -s "${BASE_URL}/api/measurements?page=1" \
  -H "User-Agent: Strong Android" \
  -H "Accept: application/json" \
  -H "x-client-build: 600013" \
  -H "x-client-platform: android" \
  | python3 -m json.tool > "${FIXTURES_DIR}/measurements_response.json"
echo "    Saved measurements_response.json"

# ---------------------------------------------------------------------------
# GET /api/users/{userId}  with logs + measurements embedded
# Uses a small limit so the fixture stays manageable.
# ---------------------------------------------------------------------------
echo "==> Fetching user (with logs embedded)..."
curl -s "${BASE_URL}/api/users/${USER_ID}?limit=5&continuation=&include=log&include=measurement" \
  -H "User-Agent: Strong Android" \
  -H "Accept: application/json" \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  -H "x-client-build: 600013" \
  -H "x-client-platform: android" \
  | python3 -m json.tool > "${FIXTURES_DIR}/user_response.json"
echo "    Saved user_response.json"

# ---------------------------------------------------------------------------
# GET /api/users/{userId}  (all include types, for a wider integration test)
# ---------------------------------------------------------------------------
echo "==> Fetching user (all includes)..."
curl -s "${BASE_URL}/api/users/${USER_ID}?limit=5&continuation=&include=log&include=measurement&include=tag&include=template&include=folder&include=widget&include=measuredValue" \
  -H "User-Agent: Strong Android" \
  -H "Accept: application/json" \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  -H "x-client-build: 600013" \
  -H "x-client-platform: android" \
  | python3 -m json.tool > "${FIXTURES_DIR}/user_response_all_includes.json"
echo "    Saved user_response_all_includes.json"

# ---------------------------------------------------------------------------
# Simulate a failed login to capture ApiErrorResponse shape
# ---------------------------------------------------------------------------
echo "==> Fetching error response (bad credentials)..."
curl -s -X POST "${BASE_URL}/auth/login" \
  -H "User-Agent: Strong Android" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -H "x-client-build: 600013" \
  -H "x-client-platform: android" \
  -d '{"usernameOrEmail": "invalid@example.com", "password": "wrongpassword"}' \
  | python3 -m json.tool > "${FIXTURES_DIR}/error_response.json"
echo "    Saved error_response.json"

echo ""
echo "All fixtures saved to ${FIXTURES_DIR}"
echo "Replace the synthetic fixture files with these real responses to make"
echo "the integration tests run against actual API data."





