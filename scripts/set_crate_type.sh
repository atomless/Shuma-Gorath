#!/bin/bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <cdylib|rlib>" >&2
  exit 1
fi

TYPE="$1"
if [[ "$TYPE" != "cdylib" && "$TYPE" != "rlib" ]]; then
  echo "Invalid crate type: $TYPE" >&2
  exit 1
fi

python3 - <<PY
from pathlib import Path

crate_type = "${TYPE}"
path = Path("Cargo.toml")
text = path.read_text()
lines = text.splitlines()
for i, line in enumerate(lines):
    if line.strip().startswith("crate-type"):
        lines[i] = f'crate-type = ["{crate_type}"]'
        break
path.write_text("\n".join(lines) + ("\n" if text.endswith("\n") else ""))
PY
