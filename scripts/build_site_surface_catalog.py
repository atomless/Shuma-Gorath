#!/usr/bin/env python3
"""Build a deterministic public-surface catalog from a local site docroot."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from site_surface_catalog import SUPPORTED_MODES, build_payload


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Compile a deterministic site-surface catalog from a local site docroot."
    )
    parser.add_argument("--docroot", required=True, help="Path to the local site docroot")
    parser.add_argument(
        "--mode",
        default="auto",
        choices=sorted(SUPPORTED_MODES),
        help="Catalog source mode (default: auto)",
    )
    parser.add_argument("--output", help="Write JSON output to this file instead of stdout")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    docroot = Path(args.docroot).expanduser().resolve()
    if not docroot.is_dir():
        raise SystemExit(f"Docroot does not exist or is not a directory: {docroot}")

    payload = build_payload(docroot, args.mode)
    rendered = json.dumps(payload, indent=2, sort_keys=True)

    if args.output:
        output_path = Path(args.output).expanduser().resolve()
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(f"{rendered}\n", encoding="utf-8")
    else:
        print(rendered)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
