"""Shared contract and build helpers for the contributor-generated /sim/public site."""

from __future__ import annotations

from pathlib import Path

from .build import build_site


SITE_DIRNAME = "sim-public-site"
MANIFEST_FILENAME = "manifest.json"
FRESHNESS_FILENAME = "freshness.json"
DEFAULT_LOCAL_STATE_DIR = ".shuma"


def artifact_root(local_state_dir: str = DEFAULT_LOCAL_STATE_DIR) -> Path:
    return Path(local_state_dir) / SITE_DIRNAME


def canonical_contract_summary() -> dict[str, object]:
    root = artifact_root()
    return {
        "generator_entrypoint": "scripts/build_sim_public_site.py",
        "generator_package": "scripts/sim_public_site",
        "corpus_policy": "config/sim_public_site/corpus.toml",
        "artifact_root": root.as_posix(),
        "manifest_path": (root / MANIFEST_FILENAME).as_posix(),
        "freshness_path": (root / FRESHNESS_FILENAME).as_posix(),
        "site_root": (root / "site").as_posix(),
        "runtime_adapter": "src/runtime/sim_public.rs",
    }


__all__ = [
    "DEFAULT_LOCAL_STATE_DIR",
    "FRESHNESS_FILENAME",
    "MANIFEST_FILENAME",
    "SITE_DIRNAME",
    "artifact_root",
    "build_site",
    "canonical_contract_summary",
]
