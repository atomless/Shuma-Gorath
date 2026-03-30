#!/bin/bash
# Shared helper functions for the repo-owned Scrapling worker runtime.

SCRAPLING_RUNTIME_VENV_DIR="${SCRAPLING_RUNTIME_VENV_DIR:-.venv-scrapling}"
SCRAPLING_RUNTIME_PACKAGE_VERSION="${SCRAPLING_RUNTIME_PACKAGE_VERSION:-0.4.3}"
SCRAPLING_RUNTIME_PACKAGE_SPEC="${SCRAPLING_RUNTIME_PACKAGE_SPEC:-scrapling[fetchers]==${SCRAPLING_RUNTIME_PACKAGE_VERSION}}"
SCRAPLING_RUNTIME_BREW_FORMULA="${SCRAPLING_RUNTIME_BREW_FORMULA:-python@3.11}"
SCRAPLING_RUNTIME_MIN_MAJOR=3
SCRAPLING_RUNTIME_MIN_MINOR=10

scrapling_runtime_venv_python() {
    printf '%s/bin/python3' "$SCRAPLING_RUNTIME_VENV_DIR"
}

scrapling_runtime_python_meets_minimum() {
    local python_bin="$1"
    "$python_bin" - <<'PY'
import sys

sys.exit(0 if sys.version_info >= (3, 10) else 1)
PY
}

scrapling_runtime_python_version() {
    local python_bin="$1"
    "$python_bin" - <<'PY'
import sys

print(f"{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}")
PY
}

scrapling_runtime_brew_python() {
    if ! command -v brew >/dev/null 2>&1; then
        return 1
    fi

    local prefix=""
    prefix="$(brew --prefix "$SCRAPLING_RUNTIME_BREW_FORMULA" 2>/dev/null || true)"
    if [[ -z "$prefix" ]]; then
        return 1
    fi

    local python_bin="$prefix/bin/python3.11"
    if [[ -x "$python_bin" ]]; then
        printf '%s\n' "$python_bin"
        return 0
    fi

    return 1
}

scrapling_runtime_select_python() {
    if command -v python3 >/dev/null 2>&1; then
        local python_bin=""
        python_bin="$(command -v python3)"
        if scrapling_runtime_python_meets_minimum "$python_bin"; then
            printf '%s\n' "$python_bin"
            return 0
        fi
    fi

    if [[ "$(uname)" == "Darwin" ]]; then
        scrapling_runtime_brew_python && return 0
    fi

    return 1
}

scrapling_runtime_install_brew_python() {
    if ! command -v brew >/dev/null 2>&1; then
        return 1
    fi

    brew install "$SCRAPLING_RUNTIME_BREW_FORMULA"
    scrapling_runtime_brew_python
}

scrapling_runtime_install() {
    local python_bin="$1"
    local venv_python=""
    venv_python="$(scrapling_runtime_venv_python)"

    if [[ -x "$venv_python" ]] && ! "$venv_python" -m pip --version >/dev/null 2>&1; then
        rm -rf "$SCRAPLING_RUNTIME_VENV_DIR"
    fi

    if [[ ! -x "$venv_python" ]]; then
        "$python_bin" -m venv "$SCRAPLING_RUNTIME_VENV_DIR"
    fi

    PIP_DISABLE_PIP_VERSION_CHECK=1 "$venv_python" -m pip install --upgrade pip
    PIP_DISABLE_PIP_VERSION_CHECK=1 "$venv_python" -m pip install --upgrade "$SCRAPLING_RUNTIME_PACKAGE_SPEC"
}

scrapling_runtime_ready() {
    local venv_python=""
    venv_python="$(scrapling_runtime_venv_python)"
    if [[ ! -x "$venv_python" ]]; then
        return 1
    fi

    "$venv_python" - <<'PY'
import importlib.metadata
import sys

if sys.version_info < (3, 10):
    raise SystemExit(1)

from scrapling.fetchers import DynamicSession, FetcherSession, StealthySession  # noqa: F401
import anyio  # noqa: F401
import curl_cffi  # noqa: F401

if importlib.metadata.version("scrapling") != "0.4.3":
    raise SystemExit(1)
PY
}

scrapling_runtime_summary() {
    local venv_python=""
    venv_python="$(scrapling_runtime_venv_python)"
    "$venv_python" - <<'PY'
import importlib.metadata
import sys

print(
    f"{sys.executable} (Python {sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}, "
    f"scrapling {importlib.metadata.version('scrapling')})"
)
PY
}
