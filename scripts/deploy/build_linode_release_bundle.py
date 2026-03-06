#!/usr/bin/env python3
"""Create a Linode deployment bundle from the exact checked-out git HEAD."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path


def run_git(repo_root: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=str(repo_root),
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or result.stdout.strip() or "git command failed")
    return result.stdout.strip()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build a tar.gz deployment bundle from the current git HEAD."
    )
    parser.add_argument("--repo-root", required=True, help="Path to local git repository root")
    parser.add_argument("--archive-output", required=True, help="Path to output tar.gz archive")
    parser.add_argument("--metadata-output", required=True, help="Path to output JSON metadata")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    repo_root = Path(args.repo_root).resolve()
    archive_output = Path(args.archive_output).resolve()
    metadata_output = Path(args.metadata_output).resolve()

    if not repo_root.exists():
        print(f"Repository root not found: {repo_root}", file=sys.stderr)
        return 2

    try:
        commit = run_git(repo_root, "rev-parse", "HEAD")
        status_output = run_git(repo_root, "status", "--porcelain")
        dirty_worktree = bool(status_output.strip())

        archive_output.parent.mkdir(parents=True, exist_ok=True)
        metadata_output.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run(
            ["git", "archive", "--format=tar.gz", "--output", str(archive_output), "HEAD"],
            cwd=str(repo_root),
            check=True,
            capture_output=True,
            text=True,
        )
        metadata_output.write_text(
            json.dumps(
                {
                    "commit": commit,
                    "dirty_worktree": dirty_worktree,
                    "repo_root": str(repo_root),
                },
                indent=2,
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )
    except RuntimeError as exc:
        print(str(exc), file=sys.stderr)
        return 2
    except subprocess.CalledProcessError as exc:
        print(exc.stderr.strip() or exc.stdout.strip() or "git archive failed", file=sys.stderr)
        return 2

    if dirty_worktree:
        print(
            "warning: local worktree is dirty; archive contains committed HEAD only",
            file=sys.stderr,
        )
    else:
        print(f"created Linode release bundle for commit {commit}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
