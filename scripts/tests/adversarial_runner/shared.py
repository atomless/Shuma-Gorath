"""Small shared helpers reused across adversarial runner helper modules."""

from __future__ import annotations

from typing import Any, Dict, List


def dict_or_empty(value: Any) -> Dict[str, Any]:
    return value if isinstance(value, dict) else {}


def list_or_empty(value: Any) -> List[Any]:
    return value if isinstance(value, list) else []


def int_or_zero(value: Any) -> int:
    try:
        return int(value)
    except Exception:
        return 0
