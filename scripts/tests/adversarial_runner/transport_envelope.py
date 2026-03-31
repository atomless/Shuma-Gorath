"""Shared header, locale, and transport-envelope helpers for adversarial realism."""

from __future__ import annotations

from typing import Any


SUPPORTED_REQUEST_CLIENT_POSTURES = {
    "desktop_browser_like",
    "mobile_browser_like",
}
SUPPORTED_BROWSER_CLIENT_POSTURES = {
    "desktop_browser_like",
    "mobile_browser_like",
}
SUPPORTED_ACCEPT_LANGUAGE_STRATEGIES = {"identity_geo_aligned"}
SUPPORTED_BROWSER_LOCALE_STRATEGIES = {"identity_geo_aligned"}
SUPPORTED_REQUEST_TRANSPORT_PROFILES = {"curl_impersonate", "urllib_direct"}
SUPPORTED_BROWSER_TRANSPORT_PROFILES = {"playwright_chromium"}

CHROME_DESKTOP_USER_AGENT = (
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 "
    "(KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
)
CHROME_ANDROID_USER_AGENT = (
    "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 "
    "(KHTML, like Gecko) Chrome/131.0.0.0 Mobile Safari/537.36"
)

_LANGUAGE_BY_COUNTRY = {
    "FR": ("fr-FR", "fr-FR,fr;q=0.9,en-US;q=0.7,en;q=0.6"),
    "GB": ("en-GB", "en-GB,en;q=0.9,en-US;q=0.7,en;q=0.6"),
    "DE": ("de-DE", "de-DE,de;q=0.9,en-US;q=0.7,en;q=0.6"),
    "US": ("en-US", "en-US,en;q=0.9"),
}

_REQUEST_TRANSPORT_REALISM = {
    "curl_impersonate": {
        "transport_realism_class": "impersonated_request_stack",
        "transport_emission_basis": "curl_cffi_impersonate",
        "transport_degraded_reason": "",
    },
    "urllib_direct": {
        "transport_realism_class": "degraded_direct_library",
        "transport_emission_basis": "python_urllib_runtime",
        "transport_degraded_reason": "no_tls_or_protocol_impersonation_support",
    },
}

_BROWSER_TRANSPORT_REALISM = {
    "playwright_chromium": {
        "transport_realism_class": "browser_runtime_stack",
        "transport_emission_basis": "playwright_chromium_runtime",
        "transport_degraded_reason": "",
    }
}


def request_transport_realism_descriptor(transport_profile: str) -> dict[str, str]:
    normalized = str(transport_profile or "").strip()
    return dict(_REQUEST_TRANSPORT_REALISM.get(normalized) or _REQUEST_TRANSPORT_REALISM["urllib_direct"])


def browser_transport_realism_descriptor(transport_profile: str) -> dict[str, str]:
    normalized = str(transport_profile or "").strip()
    return dict(
        _BROWSER_TRANSPORT_REALISM.get(normalized)
        or _BROWSER_TRANSPORT_REALISM["playwright_chromium"]
    )


def normalize_transport_envelope(raw_value: Any, *, field_name: str) -> dict[str, str]:
    if not isinstance(raw_value, dict):
        raise RuntimeError(f"{field_name} must be an object")
    request_client_posture = str(raw_value.get("request_client_posture") or "").strip()
    if request_client_posture not in SUPPORTED_REQUEST_CLIENT_POSTURES:
        raise RuntimeError(
            f"{field_name}.request_client_posture must be a supported request posture"
        )
    browser_client_posture = str(raw_value.get("browser_client_posture") or "").strip()
    if browser_client_posture not in SUPPORTED_BROWSER_CLIENT_POSTURES:
        raise RuntimeError(
            f"{field_name}.browser_client_posture must be a supported browser posture"
        )
    accept_language_strategy = str(raw_value.get("accept_language_strategy") or "").strip()
    if accept_language_strategy not in SUPPORTED_ACCEPT_LANGUAGE_STRATEGIES:
        raise RuntimeError(
            f"{field_name}.accept_language_strategy must be a supported strategy"
        )
    browser_locale_strategy = str(raw_value.get("browser_locale_strategy") or "").strip()
    if browser_locale_strategy not in SUPPORTED_BROWSER_LOCALE_STRATEGIES:
        raise RuntimeError(
            f"{field_name}.browser_locale_strategy must be a supported strategy"
        )
    request_transport_profile = str(raw_value.get("request_transport_profile") or "").strip()
    if request_transport_profile not in SUPPORTED_REQUEST_TRANSPORT_PROFILES:
        raise RuntimeError(
            f"{field_name}.request_transport_profile must be a supported request transport"
        )
    browser_transport_profile = str(raw_value.get("browser_transport_profile") or "").strip()
    if browser_transport_profile not in SUPPORTED_BROWSER_TRANSPORT_PROFILES:
        raise RuntimeError(
            f"{field_name}.browser_transport_profile must be a supported browser transport"
        )
    return {
        "request_client_posture": request_client_posture,
        "browser_client_posture": browser_client_posture,
        "accept_language_strategy": accept_language_strategy,
        "browser_locale_strategy": browser_locale_strategy,
        "request_transport_profile": request_transport_profile,
        "browser_transport_profile": browser_transport_profile,
    }


def _normalized_country_code(country_code: str | None) -> str:
    raw = str(country_code or "").strip().upper()
    if len(raw) == 2 and raw.isalpha():
        return raw
    return "US"


def _language_profile(country_code: str | None) -> tuple[str, str]:
    return _LANGUAGE_BY_COUNTRY.get(_normalized_country_code(country_code), _LANGUAGE_BY_COUNTRY["US"])


def _request_user_agent_profile(request_client_posture: str) -> dict[str, str]:
    if request_client_posture == "mobile_browser_like":
        return {
            "user_agent_family": "chrome_android",
            "user_agent": CHROME_ANDROID_USER_AGENT,
            "request_impersonate": "chrome131_android",
        }
    return {
        "user_agent_family": "chrome_desktop",
        "user_agent": CHROME_DESKTOP_USER_AGENT,
        "request_impersonate": "chrome",
    }


def _browser_user_agent_profile(browser_client_posture: str) -> dict[str, str]:
    if browser_client_posture == "mobile_browser_like":
        return {
            "user_agent_family": "chrome_android",
            "user_agent": CHROME_ANDROID_USER_AGENT,
        }
    return {
        "user_agent_family": "chrome_desktop",
        "user_agent": CHROME_DESKTOP_USER_AGENT,
    }


def resolve_request_transport_observation(
    profile: dict[str, Any],
    *,
    country_code: str | None = None,
) -> dict[str, str]:
    transport_envelope = normalize_transport_envelope(
        profile.get("transport_envelope"),
        field_name="realism_profile.transport_envelope",
    )
    _, accept_language = _language_profile(country_code)
    user_agent_profile = _request_user_agent_profile(
        transport_envelope["request_client_posture"]
    )
    transport_realism = request_transport_realism_descriptor(
        transport_envelope["request_transport_profile"]
    )
    return {
        "transport_profile": transport_envelope["request_transport_profile"],
        "transport_realism_class": transport_realism["transport_realism_class"],
        "transport_emission_basis": transport_realism["transport_emission_basis"],
        "transport_degraded_reason": transport_realism["transport_degraded_reason"],
        "user_agent_family": user_agent_profile["user_agent_family"],
        "user_agent": user_agent_profile["user_agent"],
        "request_impersonate": user_agent_profile["request_impersonate"],
        "accept_language": accept_language,
    }


def resolve_browser_transport_observation(
    profile: dict[str, Any],
    *,
    country_code: str | None = None,
) -> dict[str, str]:
    transport_envelope = normalize_transport_envelope(
        profile.get("transport_envelope"),
        field_name="realism_profile.transport_envelope",
    )
    browser_locale, accept_language = _language_profile(country_code)
    user_agent_profile = _browser_user_agent_profile(
        transport_envelope["browser_client_posture"]
    )
    transport_realism = browser_transport_realism_descriptor(
        transport_envelope["browser_transport_profile"]
    )
    return {
        "transport_profile": transport_envelope["browser_transport_profile"],
        "transport_realism_class": transport_realism["transport_realism_class"],
        "transport_emission_basis": transport_realism["transport_emission_basis"],
        "transport_degraded_reason": transport_realism["transport_degraded_reason"],
        "user_agent_family": user_agent_profile["user_agent_family"],
        "user_agent": user_agent_profile["user_agent"],
        "browser_locale": browser_locale,
        "accept_language": accept_language,
    }
