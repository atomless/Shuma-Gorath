import pathlib
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
INTEGRATION_SCRIPT = REPO_ROOT / "scripts" / "tests" / "integration.sh"
RUN_DASHBOARD_E2E_SCRIPT = REPO_ROOT / "scripts" / "tests" / "run_dashboard_e2e.sh"
MAKEFILE = REPO_ROOT / "Makefile"


class IntegrationCleanupContractTests(unittest.TestCase):
    def test_original_config_snapshot_is_captured_before_preflight_normalization(self):
        source = INTEGRATION_SCRIPT.read_text(encoding="utf-8")
        capture_marker = 'info "Capturing original runtime config snapshot for exact restore..."'
        preflight_marker = '# Preflight: normalize runtime config so tests are deterministic'
        self.assertIn(capture_marker, source)
        self.assertIn(preflight_marker, source)
        self.assertLess(source.index(capture_marker), source.index(preflight_marker))

    def test_cleanup_restores_original_snapshot_without_unconditional_override(self):
        source = INTEGRATION_SCRIPT.read_text(encoding="utf-8")
        cleanup_start = source.index("cleanup_integration_state() {")
        cleanup_end = source.index("trap cleanup_integration_state EXIT")
        cleanup_body = source[cleanup_start:cleanup_end]
        self.assertIn('if [[ -n "${ORIGINAL_CONFIG_RESTORE_PAYLOAD:-}" ]]; then', cleanup_body)
        self.assertIn('else\n    # Fallback only when the original snapshot could not be captured.', cleanup_body)
        self.assertNotIn("Ensure edge/Akamai toggles return to secure defaults", cleanup_body)

    def test_tarpit_dynamic_ips_are_part_of_cleanup_and_preflight_unban_contract(self):
        source = INTEGRATION_SCRIPT.read_text(encoding="utf-8")
        self.assertIn('TEST_TARPIT_TAMPER_IP="10.0.${TARPIT_TEST_SUBNET}.41"', source)
        self.assertIn('TARPIT_BURST_IPS=(', source)
        self.assertIn('"${TEST_TARPIT_TAMPER_IP}"', source)
        self.assertIn('"${TARPIT_BURST_IPS[@]}"', source)
        self.assertIn('info "Clearing tarpit integration test IPs..."', source)
        self.assertIn('for ip in "${TEST_TARPIT_IP}" "${TEST_TARPIT_TAMPER_IP}" "${TARPIT_BURST_IPS[@]}"; do', source)

    def test_cleanup_contract_clears_unknown_loopback_identity(self):
        source = INTEGRATION_SCRIPT.read_text(encoding="utf-8")
        cleanup_start = source.index("TEST_CLEANUP_IPS=(")
        cleanup_end = source.index(")", cleanup_start)
        cleanup_block = source[cleanup_start:cleanup_end]
        self.assertIn("unknown", cleanup_block)

    def test_full_suite_clears_loopback_bans_before_final_seed_snapshot(self):
        source = MAKEFILE.read_text(encoding="utf-8")
        test_target_start = source.index("test: ## Run the canonical local/CI pre-merge suite")
        test_target_end = source.index("\ntest-unit:", test_target_start)
        test_target_body = source[test_target_start:test_target_end]
        self.assertIn("clear-dev-loopback-bans", test_target_body)
        self.assertLess(
            test_target_body.index("clear-dev-loopback-bans"),
            test_target_body.index("seed-dashboard-data"),
        )

    def test_loopback_cleanup_target_covers_local_identity_triplet(self):
        source = MAKEFILE.read_text(encoding="utf-8")
        target_start = source.index("clear-dev-loopback-bans:")
        target_end = source.index("\nseed-dashboard-data:", target_start)
        target_body = source[target_start:target_end]
        self.assertIn("for ip in 127.0.0.1 ::1 unknown; do", target_body)
        self.assertIn('/shuma/admin/unban?ip=$$ip', target_body)

    def test_dashboard_e2e_wrapper_clears_loopback_bans_before_and_after_browser_runs(self):
        source = RUN_DASHBOARD_E2E_SCRIPT.read_text(encoding="utf-8")
        self.assertIn("cleanup_loopback_bans()", source)
        self.assertIn("make --no-print-directory clear-dev-loopback-bans", source)
        self.assertIn("trap cleanup_loopback_bans EXIT", source)
        self.assertLess(
            source.index("cleanup_loopback_bans"),
            source.index("corepack pnpm run test:dashboard:e2e:raw"),
        )

    def test_dev_start_flows_schedule_loopback_cleanup_after_runtime_boot(self):
        source = MAKEFILE.read_text(encoding="utf-8")
        self.assertIn(
            "LOCAL_LOOPBACK_BAN_CLEANUP_BACKGROUND := ( $(MAKE) --no-print-directory clear-dev-loopback-bans >/dev/null 2>&1 || true ) &",
            source,
        )
        for target_name, next_target_name in [
            ("dev: ## Build and run with file watching (auto-rebuild on save)", "\ndev-prod:"),
            ("run: ## Build once and run (no file watching)", "\nrun-prebuilt:"),
            ("run-prebuilt: ## Run Spin using prebuilt wasm (CI helper)", "\n\n#--------------------------"),
        ]:
            target_start = source.index(target_name)
            target_end = source.index(next_target_name, target_start)
            target_body = source[target_start:target_end]
            self.assertIn("LOCAL_LOOPBACK_BAN_CLEANUP_BACKGROUND", target_body)

    def test_dev_and_run_flows_route_local_browsering_through_contributor_ingress(self):
        source = MAKEFILE.read_text(encoding="utf-8")
        self.assertIn(
            "LOCAL_CONTRIBUTOR_INGRESS_ENV := SHUMA_LOCAL_CONTRIBUTOR_INGRESS_ENABLE=1",
            source,
        )
        self.assertIn(
            "SHUMA_LOCAL_CONTRIBUTOR_ORIGIN_BASE_URL=$(LOCAL_CONTRIBUTOR_ORIGIN_BASE_URL)",
            source,
        )
        self.assertIn("LOCAL_CONTRIBUTOR_ORIGIN_BASE_URL ?= http://127.0.0.1:3001", source)
        self.assertIn("LOCAL_CONTRIBUTOR_ORIGIN_LISTEN ?= 127.0.0.1:3001", source)
        for target_name, next_target_name in [
            ("dev: ## Build and run with file watching (auto-rebuild on save)", "\ndev-prod:"),
            ("run: ## Build once and run (no file watching)", "\nrun-prebuilt:"),
            ("run-prebuilt: ## Run Spin using prebuilt wasm (CI helper)", "\n\n#--------------------------"),
        ]:
            target_start = source.index(target_name)
            target_end = source.index(next_target_name, target_start)
            target_body = source[target_start:target_end]
            self.assertIn("$(LOCAL_CONTRIBUTOR_INGRESS_ENV)", target_body)
            self.assertIn("--listen $(LOCAL_CONTRIBUTOR_ORIGIN_LISTEN)", target_body)


if __name__ == "__main__":
    unittest.main()
