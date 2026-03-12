#!/usr/bin/env python3
"""Render a deployment-specific Spin manifest with explicit gateway outbound hosts."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.deploy.spin_manifest import render_fermyon_edge_manifest, render_gateway_manifest


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Render a Spin manifest with the explicit gateway upstream allowlist."
    )
    parser.add_argument("--manifest", required=True, help="Path to source Spin manifest")
    parser.add_argument("--output", required=True, help="Path to rendered Spin manifest")
    parser.add_argument("--upstream-origin", required=True, help="Gateway upstream origin")
    parser.add_argument(
        "--profile",
        choices=("shared-server", "edge-fermyon"),
        default="shared-server",
        help="Deployment profile for manifest rendering",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    manifest_path = Path(args.manifest).resolve()
    output_path = Path(args.output).resolve()

    if not manifest_path.exists():
        print(f"Manifest not found: {manifest_path}", file=sys.stderr)
        return 2

    try:
        rendered = render_gateway_manifest(
            manifest_path.read_text(encoding="utf-8"),
            args.upstream_origin,
        ) if args.profile == "shared-server" else render_fermyon_edge_manifest(
            manifest_path.read_text(encoding="utf-8"),
            args.upstream_origin,
        )
    except ValueError as exc:
        print(f"Invalid upstream origin or manifest: {exc}", file=sys.stderr)
        return 2

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(rendered, encoding="utf-8")
    print(f"rendered gateway Spin manifest: {output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
