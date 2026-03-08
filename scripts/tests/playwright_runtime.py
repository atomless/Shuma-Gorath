#!/usr/bin/env python3
"""Shared Playwright Chromium bootstrap for repo-local test paths."""

from __future__ import annotations

import os
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Optional


REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_PLAYWRIGHT_BROWSER_CACHE = REPO_ROOT / ".cache" / "ms-playwright"
_EXECUTABLE_PROBE = (
    "const { chromium } = require('@playwright/test'); "
    "process.stdout.write(chromium.executablePath() || '');"
)


@dataclass(frozen=True)
class PlaywrightRuntimeStatus:
    browser_cache: str
    chromium_executable: str
    installed_now: bool


def build_playwright_env(
    *, base_env: Optional[Dict[str, str]] = None, browser_cache: Optional[Path] = None
) -> Dict[str, str]:
    env = dict(base_env or os.environ)
    cache_path = Path(browser_cache or DEFAULT_PLAYWRIGHT_BROWSER_CACHE)
    env["PLAYWRIGHT_BROWSERS_PATH"] = str(cache_path)
    return env


def _run_playwright_command(args: list[str], *, env: Dict[str, str]) -> subprocess.CompletedProcess:
    return subprocess.run(
        args,
        text=True,
        capture_output=True,
        check=False,
        env=env,
    )


def _resolve_chromium_executable(env: Dict[str, str]) -> str:
    proc = _run_playwright_command(
        ["corepack", "pnpm", "exec", "node", "-e", _EXECUTABLE_PROBE],
        env=env,
    )
    if proc.returncode != 0:
        detail = (proc.stderr or proc.stdout or "").strip()
        raise RuntimeError(f"failed to resolve Playwright Chromium executable: {detail}")
    return str(proc.stdout or "").strip()


def ensure_playwright_chromium(
    *, base_env: Optional[Dict[str, str]] = None, browser_cache: Optional[Path] = None
) -> PlaywrightRuntimeStatus:
    cache_path = Path(browser_cache or DEFAULT_PLAYWRIGHT_BROWSER_CACHE)
    cache_path.mkdir(parents=True, exist_ok=True)
    env = build_playwright_env(base_env=base_env, browser_cache=cache_path)

    chromium_executable = _resolve_chromium_executable(env)
    if chromium_executable and os.access(chromium_executable, os.X_OK):
        return PlaywrightRuntimeStatus(
            browser_cache=str(cache_path),
            chromium_executable=chromium_executable,
            installed_now=False,
        )

    install = _run_playwright_command(
        ["corepack", "pnpm", "exec", "playwright", "install", "chromium"],
        env=env,
    )
    if install.returncode != 0:
        detail = (install.stderr or install.stdout or "").strip()
        raise RuntimeError(f"failed to install Playwright Chromium: {detail}")

    chromium_executable = _resolve_chromium_executable(env)
    if not chromium_executable or not os.access(chromium_executable, os.X_OK):
        raise RuntimeError("Playwright Chromium install completed without an executable browser")

    return PlaywrightRuntimeStatus(
        browser_cache=str(cache_path),
        chromium_executable=chromium_executable,
        installed_now=True,
    )
