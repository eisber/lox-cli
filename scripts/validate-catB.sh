#!/bin/bash
# validate-catB.sh — Run Cat B math block tests
#
# Uses curl with explicit credentials (NOT lox CLI) to control
# exactly which host gets auth requests.
#
# Prerequisites: Run validate-setup.sh first to wire VirtualIns.

set -euo pipefail

HOST="${LOX_HOST:-http://192.168.68.72}"
LOX_USER="${LOX_USER:-admin}"
LOX_PASS="${LOX_PASS:?ERROR: Set LOX_PASS environment variable}"
CREDS="${LOX_CREDS:-$LOX_USER:$LOX_PASS}"

WIRING_FILE="/tmp/validate-wiring.json"
PASS_COUNT=0
FAIL_COUNT=0

if [ ! -f "$WIRING_FILE" ]; then
  echo "ERROR: $WIRING_FILE not found. Run validate-setup.sh first."
  exit 1
fi

# Preflight: one auth check before any tests
preflight() {
    local code
    code=$(curl -s -o /dev/null -w "%{http_code}" "$HOST/jdev/cfg/api" -u "$CREDS" 2>/dev/null)
    if [[ "$code" == "403" ]]; then echo "✗ LOCKED OUT (403) — do NOT retry" >&2; exit 2; fi
    if [[ "$code" != "200" ]]; then echo "✗ Auth failed (HTTP $code)" >&2; exit 1; fi
    echo "  ✓ Auth OK"
}

set_input() {
  local vi_name="$1" value="$2"
  local encoded="${vi_name// /%20}"
  local resp http_code
  resp=$(curl -s -w '\n%{http_code}' "$HOST/jdev/sps/io/$encoded/$value" -u "$CREDS" 2>/dev/null)
  http_code=$(echo "$resp" | tail -1)
  if [[ "$http_code" == "403" ]]; then echo "⚠ LOCKED OUT — aborting" >&2; exit 2; fi
  if [[ "$http_code" != "200" ]]; then return 1; fi
}

read_output() {
  local block="$1"
  local q_uuid
  q_uuid=$(python3 -c "import json,sys; d=json.load(open(sys.argv[1])); print(d['outputs'][sys.argv[2]])" "$WIRING_FILE" "$block" 2>/dev/null)
  if [ -z "$q_uuid" ]; then echo "ERR"; return; fi
  local resp http_code
  resp=$(curl -s -w '\n%{http_code}' "$HOST/jdev/sps/io/$q_uuid/state" -u "$CREDS" 2>/dev/null)
  http_code=$(echo "$resp" | tail -1)
  if [[ "$http_code" == "403" ]]; then echo "⚠ LOCKED OUT — aborting" >&2; exit 2; fi
  if [[ "$http_code" != "200" ]]; then echo "ERR"; return; fi
  echo "$resp" | head -1 | python3 -c "import json,sys; print(json.load(sys.stdin)['LL']['value'])" 2>/dev/null
}

# Test a 2-input math block with tolerance
test_math() {
  local block="$1" i1="$2" i2="$3" expected="$4" tolerance="${5:-0.01}"
  set_input "VI_${block}_I1" "$i1"
  set_input "VI_${block}_I2" "$i2"
  sleep 0.5

  local actual
  actual=$(read_output "$block")

  # Compare with tolerance using Python
  local result
  result=$(python3 -c "
a, e, t = float('$actual'), float('$expected'), float('$tolerance')
print('PASS' if abs(a - e) <= t else 'FAIL')
print(f'{a:.4f}')
" 2>/dev/null)
  local verdict=$(echo "$result" | head -1)
  local actual_fmt=$(echo "$result" | tail -1)

  if [ "$verdict" = "PASS" ]; then
    echo "  ✓ ${block}($i1, $i2) = $actual_fmt"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "  ✗ ${block}($i1, $i2) = $actual_fmt (expected $expected)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

# Test a 1-input math block
test_math1() {
  local block="$1" input="$2" expected="$3" tolerance="${4:-0.01}"
  set_input "VI_${block}_I" "$input"
  sleep 0.5

  local actual
  actual=$(read_output "$block")
  local result
  result=$(python3 -c "
a, e, t = float('$actual'), float('$expected'), float('$tolerance')
print('PASS' if abs(a - e) <= t else 'FAIL')
print(f'{a:.4f}')
" 2>/dev/null)
  local verdict=$(echo "$result" | head -1)
  local actual_fmt=$(echo "$result" | tail -1)

  if [ "$verdict" = "PASS" ]; then
    echo "  ✓ ${block}($input) = $actual_fmt"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "  ✗ ${block}($input) = $actual_fmt (expected $expected)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

echo "═══════════════════════════════════════"
echo "  Cat B: Math Block Tests"
echo "═══════════════════════════════════════"
echo ""

preflight
echo ""

echo "── Add2 ──"
test_math CatB_Add2 3 5 8
test_math CatB_Add2 -1 1 0
test_math CatB_Add2 0 0 0

echo ""
echo "── Subtract ──"
test_math CatB_Subtract 10 3 7
test_math CatB_Subtract 0 5 -5
test_math CatB_Subtract 3 3 0

echo ""
echo "── Multiply ──"
test_math CatB_Multiply 4 5 20
test_math CatB_Multiply 0 99 0
test_math CatB_Multiply -3 4 -12

echo ""
echo "── Divide ──"
test_math CatB_Divide 10 2 5
test_math CatB_Divide 7 3 2.333 0.01
test_math CatB_Divide 0 5 0

echo ""
echo "── Modulo ──"
test_math CatB_Modulo 10 3 1
test_math CatB_Modulo 9 3 0
test_math CatB_Modulo 7 4 3

echo ""
echo "── Integer (truncate) ──"
test_math1 CatB_Integer 3.7 3
test_math1 CatB_Integer -2.3 -2
test_math1 CatB_Integer 0.9 0

echo ""
echo "═══════════════════════════════════════"
echo "  Results: $PASS_COUNT passed, $FAIL_COUNT failed"
echo "═══════════════════════════════════════"

exit $FAIL_COUNT
