#!/usr/bin/env python3
"""Contributor-generated /sim/public site builder entrypoint."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from sim_public_site import (
    artifact_root,
    build_site,
    build_site_if_stale,
    canonical_contract_summary,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Inspect or build the contributor-generated /sim/public site artifact."
    )
    parser.add_argument(
        "--print-contract",
        action="store_true",
        help="Print the canonical generator/config/artifact contract as JSON and exit.",
    )
    parser.add_argument(
        "--repo-root",
        default=str(Path(__file__).resolve().parents[1]),
        help="Repository root containing the source markdown corpus.",
    )
    parser.add_argument(
        "--artifact-root",
        default=None,
        help="Directory where the generated site artifact should be written.",
    )
    parser.add_argument(
        "--corpus-config",
        default=str(
            Path(__file__).resolve().parents[1] / "config" / "sim_public_site" / "corpus.toml"
        ),
        help="Path to the contributor-site corpus configuration file.",
    )
    parser.add_argument(
        "--site-url",
        default="http://127.0.0.1:3000",
        help="Absolute site origin used for canonical URLs.",
    )
    parser.add_argument(
        "--if-stale-hours",
        type=int,
        default=None,
        help="Only rebuild when the artifact is missing, source-stale, or older than this many hours.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.print_contract:
        print(json.dumps(canonical_contract_summary(), indent=2, sort_keys=True))
        return 0

    repo_root = Path(args.repo_root).expanduser().resolve()
    artifact_root_path = (
        Path(args.artifact_root).expanduser().resolve()
        if args.artifact_root
        else (repo_root / artifact_root().as_posix()).resolve()
    )
    corpus_config = Path(args.corpus_config).expanduser().resolve()

    build_kwargs = {
        "repo_root": repo_root,
        "artifact_root": artifact_root_path,
        "corpus_config_path": corpus_config,
        "site_url": args.site_url.rstrip("/"),
    }
    if args.if_stale_hours is None:
        build_site(**build_kwargs)
    else:
        build_site_if_stale(if_stale_hours=args.if_stale_hours, **build_kwargs)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
