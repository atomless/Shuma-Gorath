import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MAKEFILE = REPO_ROOT / "Makefile"


class AdversarySimMakeTargetTests(unittest.TestCase):
    def test_explicit_adversary_sim_target_contract_target_owns_selector_microtests(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        match = re.search(
            r"^test-adversary-sim-target-contracts:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
            source,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn("scripts/tests/test_adversary_sim_make_targets.py", body)

    def test_supervisor_wrapper_contract_target_owns_wrapper_archaeology(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        match = re.search(
            r"^test-supervisor-wrapper-contracts:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
            source,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn("scripts/tests/test_adversary_sim_supervisor.py", body)
        self.assertIn("scripts/tests/test_oversight_supervisor.py", body)

    def test_lifecycle_target_uses_current_stale_state_selectors(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        match = re.search(
            r"^test-adversary-sim-lifecycle:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
            source,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn(
            "adversary_sim_status_reports_reconciliation_required_for_stale_running_state_when_disabled",
            body,
        )
        self.assertIn(
            "adversary_sim_status_reports_previous_process_ownership_without_mutating",
            body,
        )
        self.assertNotIn("adversary_sim_status_reconciles_idle_enabled_state_to_off", body)
        self.assertNotIn(
            "adversary_sim_status_forces_off_when_run_owned_by_previous_process_instance",
            body,
        )
        self.assertIn(
            "adversary_sim_internal_beat_updates_generation_diagnostics_contract",
            body,
        )
        self.assertNotIn("scripts/tests/test_adversary_sim_supervisor.py", body)
        self.assertNotIn("scripts/tests/test_adversary_sim_make_targets.py", body)

    def test_lane_contract_target_uses_additive_lane_contract_selectors(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        match = re.search(
            r"^test-adversary-sim-lane-contract:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
            source,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn("start_and_stop_transitions_track_additive_lane_contract", body)
        self.assertIn("status_payload_exposes_additive_lane_migration_contract", body)
        self.assertIn(
            "adversary_sim_control_status_exposes_additive_lane_migration_contract",
            body,
        )

    def test_lane_selection_target_uses_control_lane_selectors(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        match = re.search(
            r"^test-adversary-sim-lane-selection:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
            source,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn(
            "adversary_sim_control_accepts_lane_selection_while_off_and_persists_desired_lane",
            body,
        )
        self.assertIn("adversary_sim_control_rejects_invalid_lane_value", body)
        self.assertIn(
            "adversary_sim_control_rejects_lane_only_idempotency_payload_mismatch",
            body,
        )
        self.assertIn(
            "adversary_sim_running_lane_selection_updates_desired_lane_without_switching_active_lane",
            body,
        )

    def test_llm_fit_target_uses_bounded_lane_contract_selectors(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        match = re.search(
            r"^test-adversarial-llm-fit:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
            source,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn(
            "adversary_sim_internal_beat_returns_llm_fulfillment_plan_for_bot_red_team_lane",
            body,
        )
        self.assertIn(
            "llm_fulfillment_plan_uses_frontier_reference_when_provider_keys_exist",
            body,
        )
        self.assertIn(
            "llm_fulfillment_plan_reports_unavailable_frontier_backend_without_provider_keys",
            body,
        )
        self.assertIn("scripts/tests/test_llm_fulfillment.py", body)

    def test_scrapling_category_fit_target_uses_bounded_selectors(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        match = re.search(
            r"^test-adversary-sim-scrapling-category-fit:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
            source,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn("observability::non_human_lane_fulfillment::tests::", body)
        self.assertIn(
            "scrapling_fulfillment_modes_cycle_across_request_native_personas",
            body,
        )
        self.assertIn(
            "admin::api::admin_config_tests::adversary_sim_internal_beat_returns_scrapling_worker_plan_and_switches_active_lane",
            body,
        )
        self.assertNotIn("scripts/tests/test_scrapling_worker.py", body)
        self.assertNotIn("scripts/tests/test_adversary_sim_supervisor.py", body)
        self.assertNotIn("scripts/tests/test_adversary_sim_make_targets.py", body)

    def test_post_sim_trigger_target_no_longer_owns_wrapper_archaeology(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        match = re.search(
            r"^test-oversight-post-sim-trigger:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
            source,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn(
            "post_sim_trigger_accepts_generation_evidence_from_previous_running_state",
            body,
        )
        self.assertIn(
            "adversary_sim_completion_triggers_post_sim_oversight_agent_once",
            body,
        )
        self.assertNotIn("scripts/tests/test_oversight_supervisor.py", body)


if __name__ == "__main__":
    unittest.main()
