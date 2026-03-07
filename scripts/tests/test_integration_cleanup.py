import pathlib
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
INTEGRATION_SCRIPT = REPO_ROOT / "scripts" / "tests" / "integration.sh"


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


if __name__ == "__main__":
    unittest.main()
