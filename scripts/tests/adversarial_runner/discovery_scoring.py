"""Frontier discovery metadata and scoring helpers for the adversarial runner."""

from __future__ import annotations

import copy
import hashlib
from pathlib import Path
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


def build_attack_plan(
    profile_name: str,
    execution_lane: str,
    base_url: str,
    scenarios: List[Dict[str, Any]],
    frontier_metadata: Dict[str, Any],
    generated_at_unix: int,
    *,
    frontier_attack_generation_contract: Dict[str, Any],
    frontier_attack_generation_contract_path: Path,
    frontier_attack_generation_contract_sha256: str,
    deterministic_attack_corpus: Dict[str, Any],
    deterministic_attack_corpus_path: Path,
    deterministic_attack_corpus_revision: str,
    deterministic_attack_corpus_taxonomy_version: str,
    sanitize_frontier_payload_fn: Callable[[Dict[str, Any]], Dict[str, Any]],
    scenario_driver_class_fn: Callable[[Dict[str, Any]], str],
    frontier_path_hint_for_scenario_fn: Callable[[Dict[str, Any]], str],
) -> Dict[str, Any]:
    contract = frontier_attack_generation_contract
    constraints = dict_or_empty(contract.get("constraints"))
    allowed_lanes = {
        str(item).strip()
        for item in list_or_empty(constraints.get("allowed_execution_lanes"))
        if str(item).strip()
    }
    if execution_lane not in allowed_lanes:
        raise RuntimeError(
            "frontier attack-generation contract forbids execution lane "
            f"{execution_lane} (allowed={sorted(allowed_lanes)})"
        )
    if not bool(constraints.get("deterministic_seed_required", True)):
        raise RuntimeError("frontier attack-generation contract must keep deterministic_seed_required=true")

    allowed_actions = dict_or_empty(contract.get("allowed_actions"))
    mutation_catalog = [
        dict_or_empty(item)
        for item in list_or_empty(allowed_actions.get("mutation_catalog"))
        if isinstance(item, dict)
    ]
    budgets = dict_or_empty(contract.get("resource_budgets"))
    max_generated_per_seed = max(1, int_or_zero(budgets.get("max_generated_candidates_per_seed")))
    max_generated_per_run = max(1, int_or_zero(budgets.get("max_generated_candidates_per_run")))
    allowed_retry_strategies = {
        str(item).strip()
        for item in list_or_empty(allowed_actions.get("allowed_retry_strategies"))
        if str(item).strip()
    }
    allowed_path_prefixes = [
        str(item).strip()
        for item in list_or_empty(allowed_actions.get("allowed_path_prefixes"))
        if str(item).strip()
    ]

    candidates: List[Dict[str, Any]] = []
    rejected_candidates: List[Dict[str, Any]] = []
    generated_candidate_count = 0

    def add_candidate(
        *,
        scenario_id: str,
        source_scenario_id: str,
        tier: str,
        driver: str,
        candidate_id: str,
        generation_kind: str,
        mutation_class: str,
        behavioral_class: str,
        novelty_score: float,
        raw_payload: Dict[str, Any],
    ) -> bool:
        try:
            sanitized_payload = sanitize_frontier_payload_fn(raw_payload)
        except Exception as exc:
            rejected_candidates.append(
                {
                    "candidate_id": candidate_id,
                    "source_scenario_id": source_scenario_id,
                    "generation_kind": generation_kind,
                    "mutation_class": mutation_class,
                    "reason_code": "sanitization_error",
                    "detail": str(exc),
                }
            )
            return False
        candidates.append(
            {
                "candidate_id": candidate_id,
                "source_scenario_id": source_scenario_id,
                "generation_kind": generation_kind,
                "mutation_class": mutation_class,
                "behavioral_class": behavioral_class,
                "novelty_score": round(clamp_score(novelty_score), 4),
                "governance_passed": True,
                "scenario_id": scenario_id,
                "tier": tier,
                "driver": driver,
                "payload": sanitized_payload,
            }
        )
        return True

    for scenario in scenarios:
        scenario_id = str(scenario.get("id") or "").strip()
        tier = str(scenario.get("tier") or "").strip()
        driver = str(scenario.get("driver") or "").strip()
        if not scenario_id:
            continue
        scenario_traffic_model = scenario.get("traffic_model")
        if not isinstance(scenario_traffic_model, dict):
            scenario_traffic_model = {}
        coverage_tags = scenario.get("coverage_tags")
        if not isinstance(coverage_tags, list) or not coverage_tags:
            coverage_tags = [scenario.get("tier"), scenario.get("driver")]
        expected_categories = scenario.get("expected_defense_categories")
        if not isinstance(expected_categories, list):
            expected_categories = []
        raw_payload = {
            "schema_version": "frontier_payload_schema.v1",
            "request_id": f"{profile_name}:{scenario.get('id')}",
            "profile": profile_name,
            "scenario": {
                "id": scenario.get("id"),
                "tier": scenario.get("tier"),
                "driver_class": scenario_driver_class_fn(scenario),
                "driver": scenario.get("driver"),
                "expected_outcome": scenario.get("expected_outcome"),
                "runtime_budget_ms": scenario.get("runtime_budget_ms"),
                "seed": scenario.get("seed"),
                "ip": scenario.get("ip"),
            },
            "traffic_model": {
                "cohort": scenario_traffic_model.get("persona", "adversarial"),
                "driver_class": scenario_driver_class_fn(scenario),
                "driver": scenario.get("driver"),
                "user_agent": scenario.get("user_agent"),
                "retry_strategy": scenario_traffic_model.get("retry_strategy", "single_attempt"),
                "cookie_behavior": scenario_traffic_model.get("cookie_behavior", "stateless"),
            },
            "target": {
                "base_url": base_url,
                "path_hint": frontier_path_hint_for_scenario_fn(scenario),
            },
            "public_crawl_content": {
                "scenario_description": scenario.get("description"),
            },
            "attack_metadata": {
                "expected_outcome": scenario.get("expected_outcome"),
                "execution_lane": execution_lane,
                "driver_class": scenario_driver_class_fn(scenario),
                "coverage_tags": coverage_tags,
                "expected_defense_categories": expected_categories,
            },
        }
        seed_candidate_id = f"cand-seed-{scenario_id}"
        raw_payload["request_id"] = f"{profile_name}:{scenario_id}:seed"
        add_candidate(
            scenario_id=scenario_id,
            source_scenario_id=scenario_id,
            tier=tier,
            driver=driver,
            candidate_id=seed_candidate_id,
            generation_kind="seed",
            mutation_class="seed",
            behavioral_class="baseline",
            novelty_score=0.0,
            raw_payload=raw_payload,
        )

        if generated_candidate_count >= max_generated_per_run:
            continue

        if mutation_catalog:
            rotation_seed = int(hashlib.sha256(scenario_id.encode("utf-8")).hexdigest()[:8], 16)
            offset = rotation_seed % len(mutation_catalog)
            rotated = mutation_catalog[offset:] + mutation_catalog[:offset]
        else:
            rotated = []

        for mutation in rotated[:max_generated_per_seed]:
            if generated_candidate_count >= max_generated_per_run:
                break
            mutation_id = str(mutation.get("id") or "").strip() or "mutation"
            mutation_class = str(mutation.get("class") or mutation_id).strip()
            behavioral_class = str(mutation.get("behavioral_class") or "mutation").strip()
            novelty_score = clamp_score(mutation.get("novelty_weight"))

            mutated_payload = copy.deepcopy(raw_payload)
            target = dict_or_empty(mutated_payload.get("target"))
            traffic_model = dict_or_empty(mutated_payload.get("traffic_model"))
            attack_metadata = dict_or_empty(mutated_payload.get("attack_metadata"))

            explicit_path = str(mutation.get("path_hint") or "").strip()
            path_suffix = str(mutation.get("path_suffix") or "").strip()
            candidate_path = explicit_path or str(target.get("path_hint") or "/")
            if path_suffix:
                candidate_path = (
                    f"{candidate_path.rstrip('/')}/{path_suffix.lstrip('/')}"
                    if candidate_path != "/"
                    else f"/{path_suffix.lstrip('/')}"
                )
            if allowed_path_prefixes and not any(
                candidate_path.startswith(prefix) for prefix in allowed_path_prefixes
            ):
                rejected_candidates.append(
                    {
                        "candidate_id": f"cand-mut-{scenario_id}-{mutation_id}",
                        "source_scenario_id": scenario_id,
                        "generation_kind": "mutation",
                        "mutation_class": mutation_class,
                        "reason_code": "path_out_of_policy",
                        "detail": candidate_path,
                    }
                )
                continue
            target["path_hint"] = candidate_path

            retry_strategy = str(mutation.get("retry_strategy") or "").strip()
            if retry_strategy:
                if allowed_retry_strategies and retry_strategy not in allowed_retry_strategies:
                    rejected_candidates.append(
                        {
                            "candidate_id": f"cand-mut-{scenario_id}-{mutation_id}",
                            "source_scenario_id": scenario_id,
                            "generation_kind": "mutation",
                            "mutation_class": mutation_class,
                            "reason_code": "retry_strategy_out_of_policy",
                            "detail": retry_strategy,
                        }
                    )
                    continue
                traffic_model["retry_strategy"] = retry_strategy

            user_agent_suffix = str(mutation.get("user_agent_suffix") or "").strip()
            if user_agent_suffix:
                base_user_agent = str(traffic_model.get("user_agent") or "").strip()
                traffic_model["user_agent"] = f"{base_user_agent} {user_agent_suffix}".strip()

            query_hint = dict_or_empty(mutation.get("query_hint"))
            if query_hint:
                target["query_hint"] = {
                    str(key): str(value)
                    for key, value in sorted(query_hint.items(), key=lambda item: str(item[0]))
                }

            mutation_candidate_id = f"cand-mut-{scenario_id}-{mutation_id}"
            mutated_payload["request_id"] = f"{profile_name}:{scenario_id}:{mutation_id}"
            attack_metadata.update(
                {
                    "generation_kind": "mutation",
                    "mutation_id": mutation_id,
                    "mutation_class": mutation_class,
                    "behavioral_class": behavioral_class,
                    "novelty_score": novelty_score,
                    "source_scenario_id": scenario_id,
                }
            )
            mutated_payload["target"] = target
            mutated_payload["traffic_model"] = traffic_model
            mutated_payload["attack_metadata"] = attack_metadata

            accepted = add_candidate(
                scenario_id=scenario_id,
                source_scenario_id=scenario_id,
                tier=tier,
                driver=driver,
                candidate_id=mutation_candidate_id,
                generation_kind="mutation",
                mutation_class=mutation_class,
                behavioral_class=behavioral_class,
                novelty_score=novelty_score,
                raw_payload=mutated_payload,
            )
            if accepted:
                generated_candidate_count += 1

    diversity_scoring = compute_frontier_diversity_scoring(
        candidates,
        frontier_metadata=frontier_metadata,
        contract=contract,
    )
    discovery_quality_metrics = compute_frontier_discovery_quality_metrics(
        candidates,
        rejected_candidates=rejected_candidates,
        frontier_metadata=frontier_metadata,
        contract=contract,
    )
    seed_candidate_count = len([row for row in candidates if str(row.get("generation_kind") or "") == "seed"])
    mutation_candidate_count = len(
        [row for row in candidates if str(row.get("generation_kind") or "") == "mutation"]
    )

    return {
        "schema_version": "attack-plan.v1",
        "generated_at_unix": generated_at_unix,
        "profile": profile_name,
        "execution_lane": execution_lane,
        "target_base_url": base_url,
        "frontier_mode": frontier_metadata.get("frontier_mode", "disabled"),
        "provider_count": frontier_metadata.get("provider_count", 0),
        "providers": frontier_metadata.get("providers", []),
        "diversity_confidence": frontier_metadata.get("diversity_confidence", "none"),
        "attack_generation_contract": {
            "contract_path": str(frontier_attack_generation_contract_path),
            "schema_version": str(contract.get("schema_version") or ""),
            "sha256": frontier_attack_generation_contract_sha256,
        },
        "deterministic_attack_corpus": {
            "contract_path": str(deterministic_attack_corpus_path),
            "schema_version": str(deterministic_attack_corpus.get("schema_version") or ""),
            "corpus_revision": deterministic_attack_corpus_revision,
            "taxonomy_version": deterministic_attack_corpus_taxonomy_version,
        },
        "generation_summary": {
            "seed_candidate_count": seed_candidate_count,
            "generated_candidate_count": mutation_candidate_count,
            "accepted_candidate_count": len(candidates),
            "rejected_candidate_count": len(rejected_candidates),
        },
        "diversity_scoring": diversity_scoring,
        "discovery_quality_metrics": discovery_quality_metrics,
        "rejected_candidates": rejected_candidates,
        "candidates": candidates,
    }
