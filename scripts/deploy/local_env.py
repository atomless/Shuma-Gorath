"""Shared helpers for gitignored local operator env files."""

from __future__ import annotations

from pathlib import Path


def strip_wrapping_quotes(value: str) -> str:
    if len(value) >= 2 and value[0] == value[-1] and value[0] in {'"', "'"}:
        return value[1:-1]
    return value


def ensure_env_file(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if not path.exists():
        path.write_text("", encoding="utf-8")
    path.chmod(0o600)


def read_env_value(path: Path, key: str) -> str:
    if not path.exists():
        return ""
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        if raw_line.startswith(f"{key}="):
            return strip_wrapping_quotes(raw_line.split("=", 1)[1])
    return ""


def upsert_env_value(path: Path, key: str, value: str) -> None:
    ensure_env_file(path)
    new_line = f"{key}={value}"
    lines = path.read_text(encoding="utf-8").splitlines() if path.exists() else []
    replaced = False
    updated: list[str] = []
    for line in lines:
        if line.startswith(f"{key}="):
            updated.append(new_line)
            replaced = True
        else:
            updated.append(line)
    if not replaced:
        updated.append(new_line)
    path.write_text("\n".join(updated).rstrip() + "\n", encoding="utf-8")
    path.chmod(0o600)
