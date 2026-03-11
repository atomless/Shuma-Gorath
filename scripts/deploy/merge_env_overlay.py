#!/usr/bin/env python3
"""Merge a KEY=value overlay into an env file without duplicating keys."""

from __future__ import annotations

import argparse
import re
from pathlib import Path


KEY_PATTERN = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=")


def merge_env_overlay(overlay_path: Path, env_path: Path) -> None:
    existing_lines = env_path.read_text(encoding="utf-8").splitlines() if env_path.exists() else []
    overlay_lines = overlay_path.read_text(encoding="utf-8").splitlines()

    overlay_updates: dict[str, str] = {}
    overlay_order: list[str] = []
    for raw_line in overlay_lines:
        if not KEY_PATTERN.match(raw_line):
            continue
        key, value = raw_line.split("=", 1)
        if key not in overlay_updates:
            overlay_order.append(key)
        overlay_updates[key] = value

    merged_lines: list[str] = []
    seen_overlay_keys: set[str] = set()
    for raw_line in existing_lines:
        if KEY_PATTERN.match(raw_line):
            key = raw_line.split("=", 1)[0]
            if key in overlay_updates:
                if key not in seen_overlay_keys:
                    merged_lines.append(f"{key}={overlay_updates[key]}")
                    seen_overlay_keys.add(key)
                continue
        merged_lines.append(raw_line)

    for key in overlay_order:
        if key not in seen_overlay_keys:
            merged_lines.append(f"{key}={overlay_updates[key]}")

    env_path.write_text("\n".join(merged_lines).rstrip("\n") + "\n", encoding="utf-8")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Merge a KEY=value overlay into an env file.")
    parser.add_argument("--overlay", required=True, help="Path to the overlay env file")
    parser.add_argument("--env-file", required=True, help="Path to the env file to update in place")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    merge_env_overlay(Path(args.overlay), Path(args.env_file))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
