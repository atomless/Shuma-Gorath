#!/usr/bin/env python3
"""Create a Linode deployment bundle from the exact checked-out git HEAD."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tarfile
import tempfile
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


def run_command(repo_root: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        list(args),
        cwd=str(repo_root),
        capture_output=True,
        text=True,
        check=False,
    )


def shuma_dashboard_build_required(repo_root: Path) -> bool:
    return (
        (repo_root / "Makefile").exists()
        and (repo_root / "spin.toml").exists()
        and (repo_root / "dashboard").is_dir()
    )


def extract_head_tree(repo_root: Path, destination: Path) -> None:
    archive_path = destination / "head.tar"
    subprocess.run(
        ["git", "archive", "--format=tar", "--output", str(archive_path), "HEAD"],
        cwd=str(repo_root),
        check=True,
        capture_output=True,
        text=True,
    )
    with tarfile.open(archive_path, "r") as archive:
        archive.extractall(destination)
    archive_path.unlink()


def maybe_build_dashboard(repo_root: Path, checkout_root: Path) -> bool:
    if not shuma_dashboard_build_required(repo_root):
        return False

    source_node_modules = repo_root / "node_modules"
    target_node_modules = checkout_root / "node_modules"
    if source_node_modules.exists() and not target_node_modules.exists():
        os.symlink(source_node_modules, target_node_modules, target_is_directory=True)

    result = run_command(checkout_root, "make", "--no-print-directory", "dashboard-build")
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip() or "dashboard-build failed"
        raise RuntimeError(f"Failed to build dashboard assets for Linode bundle: {detail}")

    dist_dashboard = checkout_root / "dist" / "dashboard"
    if not dist_dashboard.is_dir():
        raise RuntimeError(
            "Dashboard build completed without producing dist/dashboard for Linode bundle."
    )
    return True


def maybe_build_runtime(repo_root: Path, checkout_root: Path) -> bool:
    if not shuma_dashboard_build_required(repo_root):
        return False

    result = run_command(checkout_root, "make", "--no-print-directory", "build-runtime")
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip() or "build-runtime failed"
        raise RuntimeError(f"Failed to build runtime artifact for release bundle: {detail}")

    runtime_artifact = checkout_root / "dist" / "wasm" / "shuma_gorath.wasm"
    if not runtime_artifact.is_file():
        raise RuntimeError(
            "Runtime build completed without producing dist/wasm/shuma_gorath.wasm for release bundle."
        )
    return True


def write_release_archive(source_root: Path, archive_output: Path) -> None:
    archive_output.parent.mkdir(parents=True, exist_ok=True)
    with tarfile.open(archive_output, "w:gz") as archive:
        for path in sorted(source_root.rglob("*")):
            relative_path = path.relative_to(source_root)
            if relative_path.parts and relative_path.parts[0] == ".git":
                continue
            if relative_path == Path("node_modules"):
                continue
            if relative_path.parts and relative_path.parts[0] == "node_modules":
                continue
            archive.add(path, arcname=str(relative_path), recursive=False)


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
        metadata_output.parent.mkdir(parents=True, exist_ok=True)

        with tempfile.TemporaryDirectory(prefix="shuma-linode-bundle-") as temp_dir:
            staging_root = Path(temp_dir)
            extract_head_tree(repo_root, staging_root)
            dashboard_built = maybe_build_dashboard(repo_root, staging_root)
            runtime_built = maybe_build_runtime(repo_root, staging_root)
            write_release_archive(staging_root, archive_output)

        metadata_output.write_text(
            json.dumps(
                {
                    "commit": commit,
                    "dashboard_built": dashboard_built,
                    "runtime_built": runtime_built,
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
