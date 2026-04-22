#!/usr/bin/env bash
# Validate logic blocks via CLI-only pipeline (no Loxone Config UX needed)
# Usage: ./scripts/validate-logic.sh [config.Loxone]
set -euo pipefail

MS="http://192.168.68.72"
AUTH="${LOX_USER:-admin}:${LOX_PASS:?Set LOX_PASS}"

PASS=0; FAIL=0

test_2input() {
  local name=$1 output=$2 op=$3
  echo "── $name ──"
  for i1 in 0 1; do
    for i2 in 0 1; do
      curl -s -u "$AUTH" "$MS/jdev/sps/io/VI_And_I1/$i1" > /dev/null
      curl -s -u "$AUTH" "$MS/jdev/sps/io/VI_And_I2/$i2" > /dev/null
      sleep 1
      out=$(curl -s -u "$AUTH" "$MS/jdev/sps/io/$output/state" \
        | grep -o '"value": "[^"]*"' | cut -d'"' -f4)
      case "$op" in
        and) expected=$(( i1 & i2 )) ;;
        or)  expected=$(( i1 | i2 )) ;;
        xor) expected=$(( i1 ^ i2 )) ;;
      esac
      if [[ "$out" == "$expected" || "$out" == "${expected}.0" ]]; then
        echo "  $name($i1,$i2) = $out ✓"; PASS=$((PASS+1))
      else
        echo "  $name($i1,$i2) = $out (expected $expected) ✗"; FAIL=$((FAIL+1))
      fi
    done
  done
}

test_1input() {
  local name=$1 output=$2
  echo "── $name ──"
  for i1 in 0 1; do
    curl -s -u "$AUTH" "$MS/jdev/sps/io/VI_And_I1/$i1" > /dev/null
    sleep 1
    out=$(curl -s -u "$AUTH" "$MS/jdev/sps/io/$output/state" \
      | grep -o '"value": "[^"]*"' | cut -d'"' -f4)
    expected=$(( 1 - i1 ))
    if [[ "$out" == "$expected" || "$out" == "${expected}.0" ]]; then
      echo "  NOT($i1) = $out ✓"; PASS=$((PASS+1))
    else
      echo "  NOT($i1) = $out (expected $expected) ✗"; FAIL=$((FAIL+1))
    fi
  done
}

echo "╔══════════════════════════════════════════════╗"
echo "║  Logic Block Validation — CLI-only Pipeline  ║"
echo "╚══════════════════════════════════════════════╝"
echo ""
test_2input "AND" "Out_And" "and"
test_2input "OR"  "Out_Or"  "or"
test_2input "XOR" "Out_Xor" "xor"
test_1input "NOT" "Out_Not"
echo ""
echo "════════════════════════════"
echo "  Results: $PASS passed, $FAIL failed ($(( PASS + FAIL )) total)"
echo "════════════════════════════"
[ "$FAIL" -eq 0 ] || exit 1
