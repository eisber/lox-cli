#!/bin/bash
# validate-setup.sh — Create VirtualIns, wire to Cat A-F blocks, push config
#
# Usage: ./scripts/validate-setup.sh [config.xml]
#
# This script:
# 1. Downloads current config from Miniserver (or uses provided file)
# 2. Creates VirtualIn elements for each block input
# 3. Wires VirtualIn outputs to block input connectors
# 4. Pushes the modified config to Miniserver (compress + upload + reboot)
#
# The wiring map and connector UUIDs are saved to /tmp/validate-wiring.json
# for use by the test scripts.

set -euo pipefail

CONFIG="${1:-}"
WIRING_FILE="/tmp/validate-wiring.json"

# Download config if not provided
if [ -z "$CONFIG" ]; then
  echo "Downloading current config..."
  lox config download --save-as /tmp/validate-config.Loxone --extract 2>/dev/null
  CONFIG="/tmp/validate-config.Loxone.Loxone"
fi

echo "Using config: $CONFIG"

# Cat A blocks and their inputs
declare -A BLOCKS
BLOCKS[CatA_And]="I1 I2"
BLOCKS[CatA_Or]="I1 I2"
BLOCKS[CatA_Not]="I"
BLOCKS[CatA_Xor]="I1 I2"

# Cat B
BLOCKS[CatB_Add2]="I1 I2"
BLOCKS[CatB_Subtract]="I1 I2"
BLOCKS[CatB_Multiply]="I1 I2"
BLOCKS[CatB_Divide]="I1 I2"
BLOCKS[CatB_Modulo]="I1 I2"
BLOCKS[CatB_Integer]="I"

echo "{"  > "$WIRING_FILE"
echo '  "virtual_inputs": {' >> "$WIRING_FILE"
FIRST=true

for BLOCK in "${!BLOCKS[@]}"; do
  INPUTS="${BLOCKS[$BLOCK]}"
  for CONN in $INPUTS; do
    VI_TITLE="VI_${BLOCK}_${CONN}"

    # Create VirtualIn (analog for numeric inputs)
    OUTPUT=$(lox config add-virtual-in "$CONFIG" "$VI_TITLE" --analog -o json 2>/dev/null)
    VI_UUID=$(echo "$OUTPUT" | python3 -c "import json,sys; print(json.load(sys.stdin)['uuid'])")
    CONN_UUID=$(echo "$OUTPUT" | python3 -c "import json,sys; print(json.load(sys.stdin)['connector_uuid'])")

    # Wire VirtualIn.AQ → Block.ConnKey
    lox config wire-connector "$CONFIG" "${BLOCK}.${CONN}" "$CONN_UUID" -q 2>/dev/null

    if [ "$FIRST" = true ]; then
      FIRST=false
    else
      echo "," >> "$WIRING_FILE"
    fi
    printf '    "%s": {"vi_uuid": "%s", "conn_uuid": "%s"}' "$VI_TITLE" "$VI_UUID" "$CONN_UUID" >> "$WIRING_FILE"
    echo "  ✓ $VI_TITLE → ${BLOCK}.${CONN}"
  done
done

echo "" >> "$WIRING_FILE"
echo "  }," >> "$WIRING_FILE"

# Collect output connector UUIDs for monitoring
echo '  "outputs": {' >> "$WIRING_FILE"
FIRST=true

for BLOCK in "${!BLOCKS[@]}"; do
  # Get the Q connector UUID from the config
  Q_UUID=$(python3 -c "
import xml.etree.ElementTree as ET
tree = ET.parse('$CONFIG')
root = tree.getroot()
for e in root.iter('C'):
    if e.get('Title') == '$BLOCK':
        for co in e.findall('Co'):
            if co.get('K') == 'Q':
                print(co.get('U'))
                break
        break
" 2>/dev/null)

  if [ -n "$Q_UUID" ]; then
    if [ "$FIRST" = true ]; then
      FIRST=false
    else
      echo "," >> "$WIRING_FILE"
    fi
    printf '    "%s": "%s"' "$BLOCK" "$Q_UUID" >> "$WIRING_FILE"
  fi
done

echo "" >> "$WIRING_FILE"
echo "  }" >> "$WIRING_FILE"
echo "}" >> "$WIRING_FILE"

echo ""
echo "Wiring map saved to $WIRING_FILE"
echo ""

# Ask before pushing
read -p "Push config to Miniserver? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
  echo "Compressing and uploading..."
  # Use the lox CLI push pipeline
  python3.12 << PYEOF
import struct, zlib, zipfile, io, ftplib, yaml
import lz4.block

MAGIC = 0xaabbccee

# Load credentials from lox config
with open("$HOME/.lox/config.yaml") as f:
    cfg = yaml.safe_load(f)
host = cfg["host"].replace("http://", "").replace("https://", "")
user = cfg["user"]
passwd = cfg["pass"]

with open("$CONFIG", "rb") as f:
    xml_data = f.read()

compressed = lz4.block.compress(xml_data, mode='default', store_size=False)
crc = zlib.crc32(xml_data) & 0xFFFFFFFF
header = struct.pack('<IIII', MAGIC, len(compressed), len(xml_data), crc)
loxcc = header + compressed

ftp = ftplib.FTP(host)
ftp.login(user, passwd)

buf = io.BytesIO()
ftp.retrbinary('RETR /backup/sps_new.zip', buf.write)
buf.seek(0)

output = io.BytesIO()
with zipfile.ZipFile(buf, 'r') as zin:
    with zipfile.ZipFile(output, 'w', zipfile.ZIP_DEFLATED) as zout:
        for item in zin.infolist():
            if item.filename == 'sps0.LoxCC':
                zout.writestr(item, loxcc)
            else:
                zout.writestr(item, zin.read(item.filename))

output.seek(0)
ftp.storbinary('STOR /backup/sps_new.zip', output)
print("Uploaded to Miniserver")

import urllib.request
urllib.request.urlopen(f'http://{user}:{passwd}@{host}/jdev/sys/reboot', timeout=5)
print("Rebooting...")
ftp.quit()
PYEOF

  echo "Waiting 40s for reboot..."
  sleep 40
  echo "Done! Run validate-catA.sh to test."
else
  echo "Skipped push. Run manually:"
  echo "  lox config push $CONFIG"
fi
