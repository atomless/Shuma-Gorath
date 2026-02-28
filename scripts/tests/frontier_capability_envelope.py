#!/usr/bin/env python3
"""Capability-envelope signing and validation for frontier worker actions."""

from __future__ import annotations

import hashlib
import hmac
import json
import time
from typing import Any, Dict, List, Tuple


CAPABILITY_CANONICAL_PREFIX = "capability-envelope.v1"
CAPABILITY_CANONICAL_SEPARATOR = "\n"


def derive_capability_verify_key(root_secret: str, run_id: str) -> str:
    if not root_secret:
        raise RuntimeError("root_secret is required to derive capability verify key")
    message = f"frontier-capability-key:{str(run_id).strip()}"
    return hmac.new(
        str(root_secret).encode("utf-8"),
        message.encode("utf-8"),
        hashlib.sha256,
    ).hexdigest()


def capability_canonical_message(
    run_id: str,
    step_id: int,
    action_type: str,
    path: str,
    nonce: str,
    issued_at: int,
    expires_at: int,
    key_id: str,
) -> str:
    parts = [
        CAPABILITY_CANONICAL_PREFIX,
        str(run_id).strip(),
        str(int(step_id)),
        str(action_type).strip(),
        str(path).strip(),
        str(nonce).strip(),
        str(int(issued_at)),
        str(int(expires_at)),
        str(key_id).strip(),
    ]
    return CAPABILITY_CANONICAL_SEPARATOR.join(parts)


def sign_capability_envelope(
    verify_key: str,
    run_id: str,
    step_id: int,
    action_type: str,
    path: str,
    nonce: str,
    issued_at: int,
    expires_at: int,
    key_id: str,
) -> str:
    message = capability_canonical_message(
        run_id=run_id,
        step_id=step_id,
        action_type=action_type,
        path=path,
        nonce=nonce,
        issued_at=issued_at,
        expires_at=expires_at,
        key_id=key_id,
    )
    return hmac.new(
        str(verify_key).encode("utf-8"),
        message.encode("utf-8"),
        hashlib.sha256,
    ).hexdigest()


def build_action_capability_envelopes(
    root_secret: str,
    run_id: str,
    actions: List[Dict[str, Any]],
    *,
    ttl_seconds: int = 120,
    key_id: str = "sim-tag-derived-v1",
    now_unix: int | None = None,
) -> Tuple[str, List[Dict[str, Any]]]:
    now_value = int(now_unix if now_unix is not None else time.time())
    ttl_seconds = max(30, int(ttl_seconds))
    verify_key = derive_capability_verify_key(root_secret, run_id)
    envelopes: List[Dict[str, Any]] = []
    for index, action in enumerate(actions):
        step_id = index + 1
        action_type = str(action.get("action_type") or "").strip()
        path = str(action.get("path") or "").strip()
        nonce_seed = f"{run_id}:{step_id}:{action_type}:{path}:{now_value}"
        nonce = hashlib.sha256(nonce_seed.encode("utf-8")).hexdigest()[:24]
        issued_at = now_value
        expires_at = now_value + ttl_seconds
        signature = sign_capability_envelope(
            verify_key=verify_key,
            run_id=run_id,
            step_id=step_id,
            action_type=action_type,
            path=path,
            nonce=nonce,
            issued_at=issued_at,
            expires_at=expires_at,
            key_id=key_id,
        )
        envelopes.append(
            {
                "run_id": str(run_id).strip(),
                "step_id": step_id,
                "action_type": action_type,
                "path": path,
                "nonce": nonce,
                "issued_at": issued_at,
                "expires_at": expires_at,
                "key_id": str(key_id).strip(),
                "signature": signature,
            }
        )
    return verify_key, envelopes


def parse_action_capability_envelopes(raw_value: str) -> List[Dict[str, Any]]:
    text = str(raw_value or "").strip()
    if not text:
        return []
    try:
        payload = json.loads(text)
    except Exception:
        return []
    if not isinstance(payload, list):
        return []
    parsed: List[Dict[str, Any]] = []
    for entry in payload:
        if not isinstance(entry, dict):
            return []
        parsed.append(dict(entry))
    return parsed


def validate_action_capability_envelopes(
    envelopes: List[Dict[str, Any]],
    *,
    verify_key: str,
    run_id: str,
    actions: List[Dict[str, Any]],
    now_unix: int | None = None,
) -> List[str]:
    errors: List[str] = []
    now_value = int(now_unix if now_unix is not None else time.time())
    if not envelopes:
        return ["missing_capability_envelopes"]
    if len(envelopes) < len(actions):
        errors.append("capability_envelope_count_less_than_actions")

    seen_nonces = set()
    expected_count = min(len(actions), len(envelopes))
    for index in range(expected_count):
        envelope = dict(envelopes[index] or {})
        expected_action = dict(actions[index] or {})
        expected_step_id = index + 1

        nonce = str(envelope.get("nonce") or "").strip()
        if not nonce:
            errors.append(f"step_{expected_step_id}:missing_nonce")
        elif nonce in seen_nonces:
            errors.append(f"step_{expected_step_id}:nonce_replay")
        else:
            seen_nonces.add(nonce)

        envelope_run_id = str(envelope.get("run_id") or "").strip()
        if envelope_run_id != str(run_id).strip():
            errors.append(f"step_{expected_step_id}:run_id_mismatch")

        step_id = int(envelope.get("step_id") or 0)
        if step_id != expected_step_id:
            errors.append(f"step_{expected_step_id}:step_id_mismatch")

        action_type = str(envelope.get("action_type") or "").strip()
        if action_type != str(expected_action.get("action_type") or "").strip():
            errors.append(f"step_{expected_step_id}:action_type_scope_mismatch")

        path = str(envelope.get("path") or "").strip()
        if path != str(expected_action.get("path") or "").strip():
            errors.append(f"step_{expected_step_id}:path_scope_mismatch")

        issued_at = int(envelope.get("issued_at") or 0)
        expires_at = int(envelope.get("expires_at") or 0)
        if issued_at <= 0 or expires_at <= issued_at:
            errors.append(f"step_{expected_step_id}:invalid_envelope_timestamps")
        else:
            if now_value < issued_at:
                errors.append(f"step_{expected_step_id}:envelope_not_yet_valid")
            if now_value > expires_at:
                errors.append(f"step_{expected_step_id}:envelope_expired")

        key_id = str(envelope.get("key_id") or "").strip()
        signature = str(envelope.get("signature") or "").strip()
        if not key_id:
            errors.append(f"step_{expected_step_id}:missing_key_id")
        if not signature:
            errors.append(f"step_{expected_step_id}:missing_signature")
        if key_id and signature and nonce and issued_at > 0 and expires_at > issued_at:
            expected_signature = sign_capability_envelope(
                verify_key=verify_key,
                run_id=envelope_run_id,
                step_id=step_id,
                action_type=action_type,
                path=path,
                nonce=nonce,
                issued_at=issued_at,
                expires_at=expires_at,
                key_id=key_id,
            )
            if not hmac.compare_digest(signature, expected_signature):
                errors.append(f"step_{expected_step_id}:invalid_signature")

    if len(envelopes) > len(actions):
        errors.append("extra_capability_envelopes_present")

    deduped: List[str] = []
    for error in errors:
        if error not in deduped:
            deduped.append(error)
    return deduped
