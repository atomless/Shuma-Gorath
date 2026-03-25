import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MAKEFILE = REPO_ROOT / "Makefile"


class AdversarySimMakeTargetTests(unittest.TestCase):
    def test_local_scrapling_runtime_env_exports_sim_tag_secret_to_host_supervisor(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        self.assertIn(
            "SCRAPLING_LOCAL_RUNTIME_ENV := SHUMA_SIM_TELEMETRY_SECRET=$(SHUMA_SIM_TELEMETRY_SECRET)",
            source,
        )
        self.assertIn(
            "SHUMA_ADVERSARY_SIM_AVAILABLE=$(DEV_ADVERSARY_SIM_AVAILABLE) $(SCRAPLING_LOCAL_RUNTIME_ENV) SPIN_ALWAYS_BUILD=0 ./scripts/run_with_oversight_supervisor.sh",
            source,
        )

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

    def test_llm_runtime_dispatch_target_uses_typed_runtime_selectors(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        match = re.search(
            r"^test-adversarial-llm-runtime-dispatch:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
            source,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn(
            "adversary_sim_worker_result_updates_llm_runtime_generation_and_lane_diagnostics",
            body,
        )
        self.assertIn("test-adversary-sim-supervisor-unit", body)
        self.assertIn("scripts/tests/test_llm_runtime_worker.py", body)

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

    def test_scrapling_malicious_request_native_target_uses_focused_selectors(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        match = re.search(
            r"^test-adversary-sim-scrapling-malicious-request-native:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
            source,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn(
            "admin::api::admin_config_tests::adversary_sim_internal_beat_returns_scrapling_worker_plan_and_switches_active_lane",
            body,
        )
        self.assertIn(
            "scripts.tests.test_scrapling_worker.ScraplingWorkerUnitTests.test_execute_worker_plan_bulk_scraper_attempts_owned_challenge_surfaces",
            body,
        )
        self.assertIn(
            "scripts.tests.test_scrapling_worker.ScraplingWorkerUnitTests.test_execute_worker_plan_http_agent_attempts_owned_request_native_abuse_surfaces",
            body,
        )
        self.assertNotIn(
            "@$(SCRAPLING_VENV_PYTHON) -m unittest scripts/tests/test_scrapling_worker.py",
            body,
        )

    def test_scrapling_coverage_receipts_target_uses_receipt_selectors(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        match = re.search(
            r"^test-adversary-sim-scrapling-coverage-receipts:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
            source,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn("observability::scrapling_owned_surface::tests::", body)
        self.assertIn(
            "recent_sim_run_history_normalizes_scrapling_profiles_and_aggregates_observed_categories",
            body,
        )
        self.assertIn(
            "snapshot_payload_projects_recent_run_owned_surface_coverage",
            body,
        )
        self.assertIn(
            "scripts.tests.test_scrapling_worker.ScraplingWorkerUnitTests.test_execute_worker_plan_emits_signed_real_scrapling_requests_and_blocks_out_of_scope_targets",
            body,
        )
        self.assertIn(
            "scripts.tests.test_scrapling_worker.ScraplingWorkerUnitTests.test_execute_worker_plan_bulk_scraper_attempts_owned_challenge_surfaces",
            body,
        )
        self.assertIn(
            "scripts.tests.test_scrapling_worker.ScraplingWorkerUnitTests.test_execute_worker_plan_http_agent_attempts_owned_request_native_abuse_surfaces",
            body,
        )

    def test_scrapling_worker_target_includes_supervisor_transport_unit_checks(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        match = re.search(
            r"^test-adversary-sim-scrapling-worker:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
            source,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn("test-adversary-sim-supervisor-unit", body)
        self.assertIn("scripts/tests/test_adversary_sim_supervisor.py", body)

    def test_scrapling_game_loop_mainline_target_uses_current_active_mainline_gates(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        match = re.search(
            r"^test-scrapling-game-loop-mainline:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
            source,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn("test-adversary-sim-scrapling-owned-surface-contract", body)
        self.assertIn("test-adversary-sim-scrapling-malicious-request-native", body)
        self.assertIn("test-adversary-sim-scrapling-coverage-receipts", body)
        self.assertIn("test-rsi-game-mainline", body)
        self.assertIn("test-rsi-game-human-only-cycles", body)
        self.assertNotIn("test-live-feedback-loop-remote", body)
        self.assertNotIn("test-adversarial-coverage", body)

    def test_rsi_game_human_only_cycles_target_is_repeated_cycle_only(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        match = re.search(
            r"^test-rsi-game-human-only-cycles:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
            source,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn(
            "post_sim_oversight_route_repeats_human_only_cycles_with_retained_then_rolled_back_config",
            body,
        )
        self.assertNotIn("test-live-feedback-loop-remote", body)
        self.assertNotIn("adversary_sim_completion_triggers_post_sim_oversight_agent_once", body)

    def test_make_target_contract_lane_runs_selector_suite_explicitly(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        match = re.search(
            r"^test-adversary-sim-make-target-contract:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
            source,
            re.MULTILINE | re.DOTALL,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn("scripts/tests/test_adversary_sim_make_targets.py", body)


if __name__ == "__main__":
    unittest.main()
