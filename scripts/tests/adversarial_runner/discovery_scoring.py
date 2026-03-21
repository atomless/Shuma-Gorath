"""Frontier discovery metadata and scoring helpers for the adversarial runner."""

from __future__ import annotations

from typing import Any, Callable, Dict, List, Mapping

from scripts.tests.adversarial_runner.shared import dict_or_empty, int_or_zero, list_or_empty

FRONTIER_PROVIDER_SPECS = (
    {
        "provider": "openai",
        "api_key_env": "SHUMA_FRONTIER_OPENAI_API_KEY",
        "model_env": "SHUMA_FRONTIER_OPENAI_MODEL",
        "default_model": "gpt-5-mini",
    },
    {
        "provider": "anthropic",
        "api_key_env": "SHUMA_FRONTIER_ANTHROPIC_API_KEY",
        "model_env": "SHUMA_FRONTIER_ANTHROPIC_MODEL",
        "default_model": "claude-3-5-haiku-latest",
    },
    {
        "provider": "google",
        "api_key_env": "SHUMA_FRONTIER_GOOGLE_API_KEY",
        "model_env": "SHUMA_FRONTIER_GOOGLE_MODEL",
        "default_model": "gemini-2.0-flash-lite",
    },
    {
        "provider": "xai",
        "api_key_env": "SHUMA_FRONTIER_XAI_API_KEY",
        "model_env": "SHUMA_FRONTIER_XAI_MODEL",
        "default_model": "grok-3-mini",
    },
)


def build_frontier_metadata(env_reader: Callable[[str], str]) -> Dict[str, Any]:
    providers: List[Dict[str, Any]] = []
    for spec in FRONTIER_PROVIDER_SPECS:
        model_id = env_reader(spec["model_env"]) or str(spec["default_model"])
        configured = bool(env_reader(spec["api_key_env"]))
        providers.append(
            {
                "provider": str(spec["provider"]),
                "model_id": model_id,
                "configured": configured,
            }
        )

    provider_count = len([provider for provider in providers if provider["configured"]])
    if provider_count == 0:
        mode = "disabled"
        diversity_confidence = "none"
    elif provider_count == 1:
        mode = "single_provider_self_play"
        diversity_confidence = "low"
    else:
        mode = "multi_provider_playoff"
        diversity_confidence = "higher"

    advisory = ""
    if provider_count == 0:
        advisory = "No frontier provider keys are configured; run continues without frontier calls."
    elif provider_count == 1:
        advisory = (
            "Only one frontier provider key is configured; run uses reduced-diversity self-play mode."
        )

    return {
        "frontier_mode": mode,
        "provider_count": provider_count,
        "providers": providers,
        "diversity_confidence": diversity_confidence,
        "reduced_diversity_warning": provider_count == 1,
        "advisory": advisory,
    }


def frontier_path_hint_for_scenario(
    scenario: Dict[str, Any],
    *,
    deterministic_driver_path_hints: Mapping[str, str],
) -> str:
    driver_name = str(scenario.get("driver") or "").strip()
    return str(deterministic_driver_path_hints.get(driver_name) or "/")


def clamp_score(value: Any) -> float:
    try:
        parsed = float(value)
    except Exception:
        parsed = 0.0
    return max(0.0, min(1.0, parsed))


def compute_frontier_diversity_scoring(
    candidates: List[Dict[str, Any]],
    *,
    frontier_metadata: Dict[str, Any],
    contract: Dict[str, Any],
) -> Dict[str, Any]:
    generated = [
        candidate
        for candidate in candidates
        if str(candidate.get("generation_kind") or "").strip() == "mutation"
    ]
    novelty_scores = [clamp_score(candidate.get("novelty_score")) for candidate in generated]
    novelty_avg = (sum(novelty_scores) / float(len(novelty_scores)) if novelty_scores else 0.0)

    generated_behavioral = {
        str(candidate.get("behavioral_class") or "").strip()
        for candidate in generated
        if str(candidate.get("behavioral_class") or "").strip()
    }
    mutation_catalog = list_or_empty(
        dict_or_empty(dict_or_empty(contract.get("allowed_actions")).get("mutation_catalog"))
    )
    target_behavioral = {
        str(dict_or_empty(row).get("behavioral_class") or "").strip()
        for row in mutation_catalog
        if str(dict_or_empty(row).get("behavioral_class") or "").strip()
    }
    behavioral_coverage = (
        len(generated_behavioral) / float(len(target_behavioral)) if target_behavioral else 0.0
    )

    scoring = dict_or_empty(contract.get("diversity_scoring"))
    baseline = dict_or_empty(scoring.get("provider_agreement_baseline"))
    frontier_mode = str(frontier_metadata.get("frontier_mode") or "disabled")
    cross_provider = clamp_score(baseline.get(frontier_mode))
    provider_count = max(0, int_or_zero(frontier_metadata.get("provider_count")))
    if frontier_mode == "multi_provider_playoff" and provider_count > 2:
        cross_provider = clamp_score(cross_provider + min(0.15, 0.05 * (provider_count - 2)))

    weights_raw = dict_or_empty(scoring.get("weights"))
    weight_cross = max(0.0, float(weights_raw.get("cross_provider_agreement") or 0.0))
    weight_novelty = max(0.0, float(weights_raw.get("novelty") or 0.0))
    weight_behavior = max(0.0, float(weights_raw.get("behavioral_class_coverage") or 0.0))
    weight_sum = weight_cross + weight_novelty + weight_behavior
    if weight_sum <= 0.0:
        weight_cross, weight_novelty, weight_behavior = 0.4, 0.35, 0.25
        weight_sum = 1.0

    normalized_score = (
        (cross_provider * weight_cross)
        + (novelty_avg * weight_novelty)
        + (behavioral_coverage * weight_behavior)
    ) / weight_sum
    return {
        "cross_provider_agreement": round(cross_provider, 4),
        "novelty_average": round(novelty_avg, 4),
        "behavioral_class_coverage": round(behavioral_coverage, 4),
        "normalized_score": round(clamp_score(normalized_score), 4),
        "weights": {
            "cross_provider_agreement": round(weight_cross / weight_sum, 4),
            "novelty": round(weight_novelty / weight_sum, 4),
            "behavioral_class_coverage": round(weight_behavior / weight_sum, 4),
        },
        "generated_behavioral_classes": sorted(generated_behavioral),
        "target_behavioral_classes": sorted(target_behavioral),
    }


def compute_frontier_discovery_quality_metrics(
    candidates: List[Dict[str, Any]],
    *,
    rejected_candidates: List[Dict[str, Any]],
    frontier_metadata: Dict[str, Any],
    contract: Dict[str, Any],
) -> Dict[str, Any]:
    generated = [
        row for row in candidates if str(row.get("generation_kind") or "").strip() == "mutation"
    ]
    novelty_min = clamp_score(
        dict_or_empty(contract.get("novelty_expectations")).get("minimum_novelty_score")
    )
    novel_candidates = [
        row for row in generated if clamp_score(row.get("novelty_score")) >= novelty_min
    ]
    providers = list_or_empty(frontier_metadata.get("providers"))
    total_provider_slots = len(providers)
    configured = max(0, int_or_zero(frontier_metadata.get("provider_count")))
    if total_provider_slots > 0:
        provider_outage_impact = ((total_provider_slots - configured) * 100.0) / float(
            total_provider_slots
        )
    else:
        provider_outage_impact = 100.0
    return {
        "candidate_count": len(candidates),
        "generated_candidate_count": len(generated),
        "novel_candidate_count": len(novel_candidates),
        "novel_confirmed_regressions": 0,
        "false_discovery_rate_percent": None,
        "provider_outage_impact_percent": round(max(0.0, provider_outage_impact), 2),
        "rejected_candidate_count": len(rejected_candidates),
    }
