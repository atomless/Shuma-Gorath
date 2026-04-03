#!/usr/bin/env python3

import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SET_CRATE_TYPE = REPO_ROOT / "scripts" / "set_crate_type.sh"


class SetCrateTypeTests(unittest.TestCase):
    def test_noop_when_requested_crate_type_already_matches(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            cargo_toml = Path(tmp_dir) / "Cargo.toml"
            original = (
                "[package]\n"
                'name = "fixture"\n'
                'version = "0.1.0"\n'
                'edition = "2021"\n'
                "\n"
                "[lib]\n"
                'crate-type = ["rlib"]\n'
            )
            cargo_toml.write_text(original, encoding="utf-8")
            before = cargo_toml.stat()

            subprocess.run(
                [str(SET_CRATE_TYPE), "rlib"],
                cwd=tmp_dir,
                check=True,
            )

            after = cargo_toml.stat()
            self.assertEqual(cargo_toml.read_text(encoding="utf-8"), original)
            self.assertEqual(after.st_mtime_ns, before.st_mtime_ns)
            self.assertEqual(after.st_ino, before.st_ino)

    def test_rewrites_manifest_when_requested_crate_type_changes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            cargo_toml = Path(tmp_dir) / "Cargo.toml"
            cargo_toml.write_text(
                "[package]\n"
                'name = "fixture"\n'
                'version = "0.1.0"\n'
                'edition = "2021"\n'
                "\n"
                "[lib]\n"
                'crate-type = ["rlib"]\n',
                encoding="utf-8",
            )

            subprocess.run(
                [str(SET_CRATE_TYPE), "cdylib"],
                cwd=tmp_dir,
                check=True,
            )

            self.assertIn(
                'crate-type = ["cdylib"]',
                cargo_toml.read_text(encoding="utf-8"),
            )


if __name__ == "__main__":
    unittest.main()
