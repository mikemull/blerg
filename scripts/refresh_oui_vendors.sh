#!/usr/bin/env bash
# Downloads IEEE's public OUI (vendor prefix) registry and loads it into the
# oui_vendors table (see docs/schema.sql). Re-run periodically to pick up
# newly registered vendor prefixes.
set -euo pipefail

DB_URL="${1:?Usage: refresh_oui_vendors.sh <postgres-connection-string>}"

RAW_CSV="$(mktemp)"
LOAD_CSV="$(mktemp)"
trap 'rm -f "$RAW_CSV" "$LOAD_CSV"' EXIT

curl -sf https://standards-oui.ieee.org/oui/oui.csv -o "$RAW_CSV"

# IEEE's format is Registry,Assignment,Organization Name,Organization Address
# with Assignment as 6 hex chars (e.g. "0050C2"). Reshape to oui,vendor with
# the OUI in the lowercase colon-separated form MacAddr::to_string() uses.
python3 - "$RAW_CSV" > "$LOAD_CSV" <<'PYEOF'
import csv, sys

seen = set()
with open(sys.argv[1], newline='', encoding='utf-8-sig') as f:
    reader = csv.DictReader(f)
    writer = csv.writer(sys.stdout)
    writer.writerow(["oui", "vendor"])
    for row in reader:
        assignment = row["Assignment"].strip()
        if len(assignment) != 6:
            continue
        oui = ":".join(assignment[i:i + 2] for i in range(0, 6, 2)).lower()
        # A handful of very old blocks are double-registered in IEEE's own
        # data (e.g. 08:00:30); keep whichever entry we see first.
        if oui in seen:
            continue
        seen.add(oui)
        writer.writerow([oui, row["Organization Name"].strip()])
PYEOF

psql "$DB_URL" -c "TRUNCATE oui_vendors;"
psql "$DB_URL" -c "\copy oui_vendors (oui, vendor) FROM '$LOAD_CSV' WITH (FORMAT csv, HEADER true)"
