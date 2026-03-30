#!/usr/bin/env python3
"""Contributor-generated /sim/public site builder entrypoint."""

from __future__ import annotations

import argparse
import json

from sim_public_site import canonical_contract_summary


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Inspect or build the contributor-generated /sim/public site artifact."
    )
    parser.add_argument(
        "--print-contract",
        action="store_true",
        help="Print the canonical generator/config/artifact contract as JSON and exit.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.print_contract:
        print(json.dumps(canonical_contract_summary(), indent=2, sort_keys=True))
        return 0
    raise SystemExit(
        "SIM-PUBSITE-1A only freezes the generator contract. Rendering lands in SIM-PUBSITE-1B."
    )


if __name__ == "__main__":
    raise SystemExit(main())
