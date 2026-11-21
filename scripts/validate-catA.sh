#!/bin/bash
# validate-catA.sh — Cat A logic gate truth tables (14 tests)
#
# Architecture:
#   VirtualIn (set via HTTP) → Block input → Block Q output → StateV (read via HTTP)
#
# Prerequisites:
#   - Cat A blocks wired: VI1,VI2→And, VI3,VI4→Or, VI5→Not, VI6,VI7→Xor
#   - Block Q outputs wired to Virtual Status (StateV) elements
#   - Config saved to Miniserver via Loxone Config UX

set -euo pipefail

HOST="${LOX_HOST:-http://192.168.68.72}"
LOX_USER="${LOX_USER:-admin}"
LOX_PASS="${LOX_PASS:?ERROR: Set LOX_PASS environment variable}"
CREDS="${LOX_CREDS:-$LOX_USER:$LOX_PASS}"
PASS_COUNT=0; FAIL_COUNT=0

# StateV UUIDs (block Q → Virtual Status, readable via /sps/io/{uuid}/state)
AND_VS="${AND_VS:-20699539-0224-4b37-ffff234d69b98eb1}"
OR_VS="${OR_VS:-2069954d-01d1-62dd-ffff234d69b98eb1}"
NOT_VS="${NOT_VS:-20699556-020e-6eb5-ffff234d69b98eb1}"
XOR_VS="${XOR_VS:-2069955f-038b-7a99-ffff234d69b98eb1}"

set_vi() {
    local resp http_code
    resp=$(curl -s -w '\n%{http_code}' "$HOST/jdev/sps/io/Eingang%20VI$1/$2" -u "$CREDS" 2>/dev/null)
    http_code=$(echo "$resp" | tail -1)
    if [[ "$http_code" == "403" ]]; then echo "⚠ LOCKED OUT — aborting" >&2; exit 2; fi
    if [[ "$http_code" != "200" ]]; then return 1; fi
}
read_vs() {
    local resp http_code
    resp=$(curl -s -w '\n%{http_code}' "$HOST/jdev/sps/io/$1/state" -u "$CREDS" 2>/dev/null)
    http_code=$(echo "$resp" | tail -1)
    if [[ "$http_code" == "403" ]]; then echo "⚠ LOCKED OUT — aborting" >&2; exit 2; fi
    if [[ "$http_code" != "200" ]]; then return 1; fi
    echo "$resp" | head -1 | python3 -c "import json,sys; print(int(float(json.load(sys.stdin)['LL']['value'])))" 2>/dev/null
}

preflight() {
    echo "Pre-flight checks..."
    local code
    code=$(curl -s -o /dev/null -w "%{http_code}" "$HOST/jdev/cfg/api" -u "$CREDS" 2>/dev/null)
    if [[ "$code" == "403" ]]; then echo "  ✗ LOCKED OUT (403) — do NOT retry"; exit 2; fi
    if [[ "$code" != "200" ]]; then echo "  ✗ Miniserver unreachable (HTTP $code)"; exit 1; fi
    echo "  ✓ Miniserver reachable"

    local val
    val=$(read_vs "$AND_VS")
    [ -n "$val" ] && echo "  ✓ StateV readable (And=$val)" || { echo "  ✗ StateV not readable"; exit 1; }
    echo ""
}

test_gate() {
    local name=$1 vs=$2 vi1=$3 vi2=$4 v1=$5 v2=$6 expected=$7
    set_vi "$vi1" "$v1" || { echo "  ✗ $name: set_vi $vi1 failed"; FAIL_COUNT=$((FAIL_COUNT+1)); return; }
    set_vi "$vi2" "$v2" || { echo "  ✗ $name: set_vi $vi2 failed"; FAIL_COUNT=$((FAIL_COUNT+1)); return; }
    sleep 0.3
    local actual
    actual=$(read_vs "$vs")
    if [ "$actual" = "$expected" ]; then
        echo "  ✓ $name($v1,$v2) = $actual"
        PASS_COUNT=$((PASS_COUNT+1))
    else
        echo "  ✗ $name($v1,$v2) = $actual (expected $expected)"
        FAIL_COUNT=$((FAIL_COUNT+1))
    fi
}

test_not() {
    local v=$1 expected=$2
    set_vi 5 "$v"
    sleep 0.3
    local actual
    actual=$(read_vs "$NOT_VS")
    if [ "$actual" = "$expected" ]; then
        echo "  ✓ NOT($v) = $actual"
        PASS_COUNT=$((PASS_COUNT+1))
    else
        echo "  ✗ NOT($v) = $actual (expected $expected)"
        FAIL_COUNT=$((FAIL_COUNT+1))
    fi
}

echo "═══════════════════════════════════════"
echo "  Cat A: Logic Gate Truth Tables"
echo "═══════════════════════════════════════"
echo ""

preflight

echo "── AND ──"
test_gate AND "$AND_VS" 1 2 0 0 0
test_gate AND "$AND_VS" 1 2 0 1 0
test_gate AND "$AND_VS" 1 2 1 0 0
test_gate AND "$AND_VS" 1 2 1 1 1

echo ""
echo "── OR ──"
test_gate OR "$OR_VS" 3 4 0 0 0
test_gate OR "$OR_VS" 3 4 0 1 1
test_gate OR "$OR_VS" 3 4 1 0 1
test_gate OR "$OR_VS" 3 4 1 1 1

echo ""
echo "── NOT ──"
test_not 0 1
test_not 1 0

echo ""
echo "── XOR ──"
test_gate XOR "$XOR_VS" 6 7 0 0 0
test_gate XOR "$XOR_VS" 6 7 0 1 1
test_gate XOR "$XOR_VS" 6 7 1 0 1
test_gate XOR "$XOR_VS" 6 7 1 1 0

echo ""
echo "═══════════════════════════════════════"
echo "  Results: $PASS_COUNT passed, $FAIL_COUNT failed"
echo "═══════════════════════════════════════"
[ "$FAIL_COUNT" -eq 0 ] && exit 0 || exit 1
