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

python3 - "$TYPE" <<'PY'
import fcntl
import os
import stat
import sys
import tempfile
from pathlib import Path

crate_type = sys.argv[1]
path = Path("Cargo.toml")
lock_path = path.with_suffix(path.suffix + ".lock")

if not path.exists():
    raise SystemExit("Cargo.toml not found")

with lock_path.open("w") as lock_file:
    fcntl.flock(lock_file.fileno(), fcntl.LOCK_EX)

    text = path.read_text()
    if not text.strip():
        raise SystemExit("Cargo.toml is empty; refusing to mutate crate-type")

    lines = text.splitlines()
    replaced = False
    for i, line in enumerate(lines):
        if line.strip().startswith("crate-type"):
            lines[i] = f'crate-type = ["{crate_type}"]'
            replaced = True
            break

    if not replaced:
        raise SystemExit("crate-type entry not found in Cargo.toml")

    payload = "\n".join(lines) + ("\n" if text.endswith("\n") else "")
    original_mode = stat.S_IMODE(path.stat().st_mode)

    fd, tmp_path = tempfile.mkstemp(
        prefix=".Cargo.toml.",
        suffix=".tmp",
        dir=str(path.parent),
    )
    try:
        os.chmod(tmp_path, original_mode)
        with os.fdopen(fd, "w") as tmp_file:
            tmp_file.write(payload)
            tmp_file.flush()
            os.fsync(tmp_file.fileno())
        os.replace(tmp_path, path)
    finally:
        if os.path.exists(tmp_path):
            os.unlink(tmp_path)
PY
