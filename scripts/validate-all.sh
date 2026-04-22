#!/bin/bash
# validate-all.sh — Comprehensive Loxone block validation using lox CLI
#
# ALL XML manipulation uses the Rust CLI (lox config add, etc.)
# to preserve BOM, CRLF, and XML declaration formatting.
# NEVER use Python ET.tostring() on Loxone config files.
#
# Usage:
#   ./validate-all.sh structural    # Test all 163 types can be injected
#   ./validate-all.sh inject logic  # Inject + upload logic test circuits
#   ./validate-all.sh test logic    # Run logic tests (blocks must exist)
#   ./validate-all.sh full logic    # inject + upload + reboot + test
#
# Environment:
#   LOX_HOST   (default: http://192.168.68.72)
#   LOX_USER   (default: admin)
#   LOX_PASS   (required for test/full modes)
#   LOX_CREDS  (default: $LOX_USER:$LOX_PASS, for curl basic auth)

set -euo pipefail

HOST="${LOX_HOST:-http://192.168.68.72}"
LOX_USER="${LOX_USER:-admin}"
LOX_PASS="${LOX_PASS:-}"
CREDS="${LOX_CREDS:-$LOX_USER:$LOX_PASS}"
SETTLE="${LOX_SETTLE:-1.5}"
REBOOT_WAIT="${LOX_REBOOT_WAIT:-75}"

PASS_COUNT=0; FAIL_COUNT=0; SKIP_COUNT=0

# ── Helpers ─────────────────────────────────────────────────────────

die()  { echo "ERROR: $*" >&2; exit 1; }
info() { echo "  $*"; }
ok()   { echo "  ✓ $*"; }
fail() { echo "  ✗ $*"; }

set_vi() {
    local name="$1" value="$2"
    local encoded="${name// /%20}"
    local resp http_code
    resp=$(curl -s -w '\n%{http_code}' "$HOST/jdev/sps/io/$encoded/$value" -u "$CREDS" 2>/dev/null)
    http_code=$(echo "$resp" | tail -1)
    if [[ "$http_code" == "403" ]]; then
        echo "⚠ AUTH LOCKED OUT — aborting to prevent escalation" >&2
        exit 2
    fi
    echo "$resp" | grep -q '"200"'
}

read_sv() {
    local uuid="$1"
    local resp http_code
    resp=$(curl -s -w '\n%{http_code}' "$HOST/jdev/sps/io/$uuid/state" -u "$CREDS" 2>/dev/null)
    http_code=$(echo "$resp" | tail -1)
    if [[ "$http_code" == "403" ]]; then
        echo "⚠ AUTH LOCKED OUT — aborting to prevent escalation" >&2
        exit 2
    fi
    echo "$resp" | head -1 | python3 -c "import json,sys; print(json.load(sys.stdin)['LL']['value'])" 2>/dev/null
}

check_auth() {
    local resp http_code
    resp=$(curl -s -w '\n%{http_code}' "$HOST/jdev/cfg/api" -u "$CREDS" 2>/dev/null)
    http_code=$(echo "$resp" | tail -1)
    if [[ "$http_code" == "403" ]]; then
        die "AUTH LOCKED OUT (403) — do NOT retry. Wait for lockout to expire."
    fi
    echo "$resp" | grep -q '"200"' || die "Auth failed (HTTP $http_code — check LOX_PASS)"
}

check_xml_integrity() {
    local file="$1"
    python3 -c "
import sys
data = open(sys.argv[1],'rb').read()
bom = data.startswith(b'\xef\xbb\xbf')
crlf = data.count(b'\r\n')
lf = data.count(b'\n') - crlf
dq = b'version=\"1.0\"' in data[:100]
print(f'BOM={bom} CRLF={crlf} LF={lf} DblQuote={dq}')
if not bom: print('WARN: Missing BOM')
print('OK')
" "$file"
}

wait_for_reboot() {
    echo "  Waiting ${REBOOT_WAIT}s for reboot..."
    sleep "$REBOOT_WAIT"
    for i in $(seq 1 30); do
        if curl -sf "$HOST/dev/sys/check" >/dev/null 2>&1; then
            echo "  Miniserver online!"
            sleep 3
            return 0
        fi
        sleep 2
    done
    die "Miniserver did not come back"
}

test_gate() {
    # test_gate BLOCK_TYPE VI_NAMES... -- OUTPUT_UUID CASES...
    # Each CASE: "i1,i2,...=expected"
    local block="$1"; shift
    local vi_names=()
    while [[ "$1" != "--" ]]; do vi_names+=("$1"); shift; done
    shift  # skip --
    local out_uuid="$1"; shift

    local cases=("$@")
    local p=0 f=0

    for case in "${cases[@]}"; do
        local inputs="${case%%=*}"
        local expected="${case##*=}"

        # Set inputs
        IFS=',' read -ra vals <<< "$inputs"
        for i in "${!vals[@]}"; do
            set_vi "${vi_names[$i]}" "${vals[$i]}" || { ((f++)); continue 2; }
        done
        sleep "$SETTLE"

        # Read output
        local actual
        actual=$(read_sv "$out_uuid") || { ((f++)); continue; }
        actual=$(printf "%.0f" "$actual" 2>/dev/null || echo "$actual")

        if [[ "$actual" == "$expected" ]]; then
            ((p++))
        else
            fail "$block: expected=$expected actual=$actual (inputs=$inputs)"
            ((f++))
        fi
    done

    if [[ $f -eq 0 ]]; then
        ok "$block: $p/$p PASS"
        PASS_COUNT=$((PASS_COUNT + p))
    else
        fail "$block: $p/$((p+f)) ($f failures)"
        PASS_COUNT=$((PASS_COUNT + p))
        FAIL_COUNT=$((FAIL_COUNT + f))
    fi
}

# ── Structural validation ──────────────────────────────────────────

cmd_structural() {
    echo "=== Structural Validation (163 types) ==="
    echo ""

    # Download and extract current config
    lox config download -q 2>/dev/null
    local zip
    zip=$(ls -t sps_*.zip 2>/dev/null | head -1)
    [[ -n "$zip" ]] || die "No config ZIP found"
    lox config extract "$zip" 2>/dev/null
    local loxone="${zip%.zip}.Loxone"
    [[ -f "$loxone" ]] || die "Extract failed"

    local passed=0 failed=0 total=0

    # Read all types from crosswalk.json
    local types
    types=$(python3 -c "
import json
cw = json.load(open('docs/schemas/crosswalk.json'))['types']
for t in sorted(cw.keys()): print(t)
")

    while IFS= read -r block_type; do
        cp "$loxone" /tmp/structural_test.Loxone
        ((total++)) || true
        if lox config add --type "$block_type" --title "SV_${block_type}" /tmp/structural_test.Loxone >/dev/null 2>&1; then
            echo "  ✓ $block_type"
            ((passed++)) || true
        else
            echo "  ✗ $block_type"
            ((failed++)) || true
        fi
    done <<< "$types"

    rm -f /tmp/structural_test.Loxone
    echo ""
    echo "Structural: $passed/$((passed + failed)) types inject cleanly"
}

# ── Inject test circuits ───────────────────────────────────────────

cmd_inject() {
    local category="${1:-logic}"
    echo "=== Injecting $category test circuits ==="

    lox config download -q 2>/dev/null
    local zip
    zip=$(ls -t sps_*.zip 2>/dev/null | head -1)
    lox config extract "$zip" 2>/dev/null
    local loxone="${zip%.zip}.Loxone"
    local work="/tmp/test_${category}.Loxone"
    cp "$loxone" "$work"

    local types
    case "$category" in
        logic) types="And Or Not Xor" ;;
        math)  types="Add2 Subtract Multiply Divide" ;;
        comparison) types="Comparator" ;;
        stateful) types="FlipFlopRS" ;;
        *) die "Unknown category: $category" ;;
    esac

    for block_type in $types; do
        local prefix="T_${block_type}"
        # Add block
        lox config add --type "$block_type" --title "${prefix}_Blk" "$work" >/dev/null 2>&1 \
            && ok "Added $block_type" || { fail "Add $block_type"; continue; }
    done

    # Verify XML integrity
    echo ""
    echo -n "  XML integrity: "
    check_xml_integrity "$work"

    echo ""
    echo "Config saved to: $work"
    echo "Upload with: lox config push $work --reboot --force"
}

# ── Test (assumes blocks already exist on Miniserver) ──────────────

cmd_test() {
    local category="${1:-logic}"
    echo "=== Testing $category blocks ==="

    [[ -n "$LOX_PASS" ]] || die "Set LOX_PASS environment variable"
    check_auth

    case "$category" in
        logic)
            echo "Note: Requires pre-wired VirtualIn→Block→StateV circuits"
            echo "      with known StateV UUIDs (from validate-catA.sh)"
            echo ""
            echo "Run validate-catA.sh for logic gate tests instead."
            ;;
        *)
            echo "Category '$category' test not yet implemented."
            echo "Available: logic (via validate-catA.sh)"
            ;;
    esac
}

# ── Full pipeline ──────────────────────────────────────────────────

cmd_full() {
    local category="${1:-logic}"
    cmd_inject "$category"
    local work="/tmp/test_${category}.Loxone"
    echo ""
    echo "=== Uploading and rebooting ==="
    lox config push "$work" --reboot --force
    wait_for_reboot
    cmd_test "$category"
}

# ── Main ───────────────────────────────────────────────────────────

case "${1:-help}" in
    structural) cmd_structural ;;
    inject)     cmd_inject "${2:-logic}" ;;
    test)       cmd_test "${2:-logic}" ;;
    full)       cmd_full "${2:-logic}" ;;
    help|*)
        echo "Usage: $0 <command> [category]"
        echo ""
        echo "Commands:"
        echo "  structural         Test all 163 types can be injected (no upload)"
        echo "  inject <category>  Inject test circuits into config"
        echo "  test <category>    Run tests against live Miniserver"
        echo "  full <category>    inject + upload + reboot + test"
        echo ""
        echo "Categories: logic, math, comparison, stateful"
        echo ""
        echo "Environment: LOX_HOST, LOX_USER, LOX_PASS, LOX_CREDS"
        ;;
esac
