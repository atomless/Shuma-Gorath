#!/usr/bin/env python3

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import scripts.tests.check_sim2_adr_conformance as conformance


class Sim2AdrConformanceUnitTests(unittest.TestCase):
    def test_check_markers_returns_missing_entries(self):
        text = "alpha beta gamma"
        missing = conformance.check_markers(text, ["alpha", "delta"])
        self.assertEqual(missing, ["delta"])

    def test_evaluate_requirements_reports_missing_markers(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            target = root / "docs/adr/test.md"
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text("present-marker", encoding="utf-8")
            with patch.object(
                conformance,
                "ADR_REQUIREMENTS",
                [{"id": "adr_test", "path": "docs/adr/test.md", "markers": ["present-marker", "missing"]}],
            ), patch.object(conformance, "IMPLEMENTATION_REQUIREMENTS", []):
                payload = conformance.evaluate_requirements(root)
        self.assertFalse(payload["status"]["passed"])
        self.assertEqual(len(payload["status"]["failures"]), 1)
        self.assertIn("missing markers", payload["status"]["failures"][0])

    def test_evaluate_requirements_passes_when_all_markers_present(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            target = root / "src/admin/api.rs"
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text("marker-a marker-b", encoding="utf-8")
            with patch.object(conformance, "ADR_REQUIREMENTS", []), patch.object(
                conformance,
                "IMPLEMENTATION_REQUIREMENTS",
                [{"id": "impl_test", "path": "src/admin/api.rs", "markers": ["marker-a", "marker-b"]}],
            ):
                payload = conformance.evaluate_requirements(root)
        self.assertTrue(payload["status"]["passed"])
        self.assertEqual(payload["status"]["failures"], [])


if __name__ == "__main__":
    unittest.main()
