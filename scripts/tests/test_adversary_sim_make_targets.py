import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MAKEFILE = REPO_ROOT / "Makefile"


class AdversarySimMakeTargetTests(unittest.TestCase):
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


if __name__ == "__main__":
    unittest.main()
