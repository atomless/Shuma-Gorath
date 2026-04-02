#!/usr/bin/env python3
"""Shared simulation-tag contract helpers for host-side workers and tests."""

from __future__ import annotations

import hashlib
import hmac
import json
from pathlib import Path
from typing import Any


SIM_TAG_CONTRACT_PATH = Path(__file__).resolve().parent / "adversarial" / "sim_tag_contract.v1.json"


def load_sim_tag_contract(path: Path = SIM_TAG_CONTRACT_PATH) -> dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError("sim tag contract must be a JSON object")
    return payload


SIM_TAG_CONTRACT = load_sim_tag_contract()
SIM_TAG_HEADERS = dict(SIM_TAG_CONTRACT.get("headers") or {})
SIM_TAG_HEADER_RUN_ID = SIM_TAG_HEADERS.get("sim_run_id", "x-shuma-sim-run-id")
SIM_TAG_HEADER_PROFILE = SIM_TAG_HEADERS.get("sim_profile", "x-shuma-sim-profile")
SIM_TAG_HEADER_LANE = SIM_TAG_HEADERS.get("sim_lane", "x-shuma-sim-lane")
SIM_TAG_HEADER_TIMESTAMP = SIM_TAG_HEADERS.get("sim_timestamp", "x-shuma-sim-ts")
SIM_TAG_HEADER_NONCE = SIM_TAG_HEADERS.get("sim_nonce", "x-shuma-sim-nonce")
SIM_TAG_HEADER_SIGNATURE = SIM_TAG_HEADERS.get("sim_signature", "x-shuma-sim-signature")
SIM_TAG_CANONICAL_PREFIX = str((SIM_TAG_CONTRACT.get("canonical") or {}).get("prefix", "sim-tag.v1"))
SIM_TAG_CANONICAL_SEPARATOR = str(
    (SIM_TAG_CONTRACT.get("canonical") or {}).get("separator", "\n")
)
SIM_TAG_NONCE_CHARS = 24


def build_sim_tag_canonical_message(
    run_id: str, profile: str, lane: str, timestamp: str, nonce: str
) -> str:
    parts = [
        SIM_TAG_CANONICAL_PREFIX,
        str(run_id).strip(),
        str(profile).strip(),
        str(lane).strip(),
        str(timestamp).strip(),
        str(nonce).strip(),
    ]
    return SIM_TAG_CANONICAL_SEPARATOR.join(parts)


def sign_sim_tag(
    secret: str, run_id: str, profile: str, lane: str, timestamp: str, nonce: str
) -> str:
    message = build_sim_tag_canonical_message(run_id, profile, lane, timestamp, nonce)
    return hmac.new(
        str(secret).encode("utf-8"),
        message.encode("utf-8"),
        hashlib.sha256,
    ).hexdigest()


def build_sim_tag_nonce(*parts: Any, length: int = SIM_TAG_NONCE_CHARS) -> str:
    raw = ":".join(str(part).strip() for part in parts if str(part).strip())
    return hashlib.sha256(raw.encode("utf-8")).hexdigest()[: max(8, int(length))]
