# Cat A Manual Wiring Guide

The wiring is already present in the Miniserver XML config. You just need to
**download from Miniserver → upload back** in Loxone Config to compile the SPS.

## Steps (< 2 minutes)

1. Open **Loxone Config** (should already be open and connected)
2. Click the **↓ download button** ("Laden vom Miniserver") in the connection panel
3. Wait for download to complete
4. Click the **↑ upload button** ("Speichern im Miniserver") in the connection panel
5. Confirm any prompts
6. Wait for upload + reboot (~30s)

That's it! The wiring is already in the XML — the upload step compiles it into the SPS program.

## What's Wired

| Virtual Input | → Block Input | Purpose |
|---------------|---------------|---------|
| Eingang VI1.Q | → CatA_And.I1 | AND gate input 1 |
| Eingang VI2.Q | → CatA_And.I2 | AND gate input 2 |
| Eingang VI3.Q | → CatA_Or.I1 | OR gate input 1 |
| Eingang VI4.Q | → CatA_Or.I2 | OR gate input 2 |
| Eingang VI5.Q | → CatA_Not.I | NOT gate input |
| Eingang VI6.Q | → CatA_Xor.I1 | XOR gate input 1 |
| Eingang VI7.Q | → CatA_Xor.I2 | XOR gate input 2 |

## If UX Strips the Wiring

If the download removes our XML-injected wiring, manually wire in Loxone Config:

1. Navigate to **DG Schlafzimmer** page (where blocks are)
2. For each connection above: drag from the Virtual Input's output dot to the block's input dot
3. Save to Miniserver (↑)

## After Upload

Run the automated test:
```bash
lox ws-monitor "206966b7-01f6-4cec-ffff6437a6164046,206966b7-01f6-4cf3-ffff6437a6164046,206966b7-01f6-4cf8-ffff6437a6164046,206966b7-01f6-4cff-ffff6437a6164046" --timeout 30
```

In another terminal:
```bash
# AND: 1,1 → 1
curl -s "http://$LOX_HOST/jdev/sps/io/Eingang%20VI1/1" -u "$LOX_CREDS"
curl -s "http://$LOX_HOST/jdev/sps/io/Eingang%20VI2/1" -u "$LOX_CREDS"
```
