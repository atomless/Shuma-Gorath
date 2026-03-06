#!/usr/bin/env python3
"""Select a deterministic non-reserved public path for gateway smoke forwarding checks."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.deploy.gateway_surface_catalog import load_catalog_paths, select_forward_probe_path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Select a deterministic forward-probe path from a gateway surface catalog."
    )
    parser.add_argument("--catalog", required=True, help="Path to the gateway surface catalog JSON")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        paths = load_catalog_paths(args.catalog)
        print(select_forward_probe_path(paths))
        return 0
    except (OSError, ValueError) as exc:
        print(str(exc), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
