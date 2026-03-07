import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts.deploy import remote_target


class RemoteTargetTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = Path(tempfile.mkdtemp(prefix="remote-target-"))
        self.env_file = self.temp_dir / ".env.local"
        self.receipts_dir = self.temp_dir / ".spin" / "remotes"
        self.receipts_dir.mkdir(parents=True, exist_ok=True)
        self.receipt_path = self.receipts_dir / "blog-prod.json"
        (self.temp_dir / "catalog.json").write_text('{"inventory":[{"path":"/"}]}\n', encoding="utf-8")
        remote_target.write_json(
            self.receipt_path,
            {
                "schema": "shuma.remote_target.v1",
                "identity": {
                    "name": "blog-prod",
                    "backend_kind": "ssh_systemd",
                    "provider_kind": "linode",
                },
                "ssh": {
                    "host": "198.51.100.24",
                    "port": 22,
                    "user": "shuma",
                    "private_key_path": "/Users/test/.ssh/shuma-linode",
                },
                "runtime": {
                    "app_dir": "/opt/shuma-gorath",
                    "service_name": "shuma-gorath",
                    "public_base_url": "https://blog.example.com",
                },
                "deploy": {
                    "spin_manifest_path": "/opt/shuma-gorath/spin.gateway.toml",
                    "surface_catalog_path": str(self.temp_dir / "catalog.json"),
                    "smoke_path": "/health",
                },
                "metadata": {
                    "last_deployed_commit": "abc123",
                    "last_deployed_at_utc": "2026-03-07T12:00:00Z",
                },
                "provider": {
                    "linode": {
                        "instance_id": 123456,
                    }
                },
            },
        )

    def test_use_command_persists_active_remote_in_env_file(self) -> None:
        rc = remote_target.main(
            [
                "--env-file",
                str(self.env_file),
                "--receipts-dir",
                str(self.receipts_dir),
                "use",
                "--name",
                "blog-prod",
            ]
        )

        self.assertEqual(rc, 0)
        self.assertIn("SHUMA_ACTIVE_REMOTE=blog-prod", self.env_file.read_text(encoding="utf-8"))

    def test_status_uses_active_remote_from_env_file(self) -> None:
        self.env_file.write_text("SHUMA_ACTIVE_REMOTE=blog-prod\n", encoding="utf-8")

        with patch.object(subprocess, "run") as run:
            run.return_value = subprocess.CompletedProcess(args=[], returncode=0)
            rc = remote_target.main(
                [
                    "--env-file",
                    str(self.env_file),
                    "--receipts-dir",
                    str(self.receipts_dir),
                    "status",
                ]
            )

        self.assertEqual(rc, 0)
        ssh_command = run.call_args.args[0]
        self.assertEqual(
            ssh_command,
            [
                "ssh",
                "-o",
                "StrictHostKeyChecking=accept-new",
                "-p",
                "22",
                "-i",
                "/Users/test/.ssh/shuma-linode",
                "shuma@198.51.100.24",
                "sudo systemctl status shuma-gorath --no-pager",
            ],
        )

    def test_logs_uses_expected_journalctl_command(self) -> None:
        with patch.object(subprocess, "run") as run:
            run.return_value = subprocess.CompletedProcess(args=[], returncode=0)
            rc = remote_target.main(
                [
                    "--env-file",
                    str(self.env_file),
                    "--receipts-dir",
                    str(self.receipts_dir),
                    "logs",
                    "--name",
                    "blog-prod",
                ]
            )

        self.assertEqual(rc, 0)
        self.assertEqual(
            run.call_args.args[0][-1],
            "sudo journalctl -u shuma-gorath -n 200 --no-pager",
        )

    def test_start_and_stop_use_systemctl(self) -> None:
        for command_name, expected in (
            ("start", "sudo systemctl start shuma-gorath"),
            ("stop", "sudo systemctl stop shuma-gorath"),
        ):
            with self.subTest(command_name=command_name):
                with patch.object(subprocess, "run") as run:
                    run.return_value = subprocess.CompletedProcess(args=[], returncode=0)
                    rc = remote_target.main(
                        [
                            "--env-file",
                            str(self.env_file),
                            "--receipts-dir",
                            str(self.receipts_dir),
                            command_name,
                            "--name",
                            "blog-prod",
                        ]
                    )

                self.assertEqual(rc, 0)
                self.assertEqual(run.call_args.args[0][-1], expected)

    def test_open_dashboard_uses_local_opener(self) -> None:
        with patch.object(remote_target.shutil, "which", side_effect=["/usr/bin/open", None]), patch.object(
            subprocess, "run"
        ) as run:
            run.return_value = subprocess.CompletedProcess(args=[], returncode=0)
            rc = remote_target.main(
                [
                    "--env-file",
                    str(self.env_file),
                    "--receipts-dir",
                    str(self.receipts_dir),
                    "open-dashboard",
                    "--name",
                    "blog-prod",
                ]
            )

        self.assertEqual(rc, 0)
        self.assertEqual(run.call_args.args[0], ["open", "https://blog.example.com/dashboard"])

    def test_invalid_receipt_schema_fails_cleanly(self) -> None:
        broken_path = self.receipts_dir / "broken.json"
        broken_path.write_text(json.dumps({"schema": "wrong"}) + "\n", encoding="utf-8")

        with self.assertRaises(SystemExit) as exc:
            remote_target.main(
                [
                    "--env-file",
                    str(self.env_file),
                    "--receipts-dir",
                    str(self.receipts_dir),
                    "status",
                    "--name",
                    "broken",
                ]
            )

        self.assertIn("schema", str(exc.exception))

    def test_update_builds_uploads_restarts_smokes_and_refreshes_receipt_metadata(self) -> None:
        self.env_file.write_text("SHUMA_ACTIVE_REMOTE=blog-prod\n", encoding="utf-8")
        bundle_dir = self.temp_dir / "bundle"
        bundle_dir.mkdir()
        archive_path = bundle_dir / "release.tar.gz"
        archive_path.write_text("bundle\n", encoding="utf-8")
        metadata_path = bundle_dir / "release.json"
        metadata_path.write_text(
            json.dumps(
                {
                    "commit": "deadbeef",
                    "dirty_worktree": False,
                }
            )
            + "\n",
            encoding="utf-8",
        )
        update_script_path = bundle_dir / "remote-update.sh"
        update_script_path.write_text("#!/bin/sh\n", encoding="utf-8")

        with patch.object(
            remote_target,
            "build_release_bundle",
            return_value=(archive_path, metadata_path, {"commit": "deadbeef", "dirty_worktree": False}),
        ), patch.object(
            remote_target, "write_remote_update_script", return_value=update_script_path
        ), patch.object(
            remote_target, "copy_file_to_remote"
        ) as copy_file, patch.object(
            remote_target, "run_remote_update_install", return_value=0
        ) as run_install, patch.object(
            remote_target, "run_remote_smoke", return_value=0
        ) as run_smoke:
            rc = remote_target.main(
                [
                    "--env-file",
                    str(self.env_file),
                    "--receipts-dir",
                    str(self.receipts_dir),
                    "update",
                ]
            )

        self.assertEqual(rc, 0)
        copied_remote_paths = [call.args[2] for call in copy_file.call_args_list]
        self.assertEqual(
            copied_remote_paths,
            [
                "/tmp/shuma-remote-update-release.tar.gz",
                "/tmp/shuma-remote-update-release.json",
                "/tmp/shuma-remote-update-surface-catalog.json",
                "/tmp/shuma-remote-update.sh",
            ],
        )
        run_install.assert_called_once()
        run_smoke.assert_called_once()
        receipt = json.loads(self.receipt_path.read_text(encoding="utf-8"))
        self.assertEqual(receipt["metadata"]["last_deployed_commit"], "deadbeef")
        self.assertTrue(receipt["metadata"]["last_deployed_at_utc"].endswith("Z"))

    def test_update_attempts_rollback_when_smoke_fails(self) -> None:
        self.env_file.write_text("SHUMA_ACTIVE_REMOTE=blog-prod\n", encoding="utf-8")
        bundle_dir = self.temp_dir / "bundle"
        bundle_dir.mkdir()
        archive_path = bundle_dir / "release.tar.gz"
        archive_path.write_text("bundle\n", encoding="utf-8")
        metadata_path = bundle_dir / "release.json"
        metadata_path.write_text(
            json.dumps(
                {
                    "commit": "deadbeef",
                    "dirty_worktree": False,
                }
            )
            + "\n",
            encoding="utf-8",
        )
        update_script_path = bundle_dir / "remote-update.sh"
        update_script_path.write_text("#!/bin/sh\n", encoding="utf-8")

        with patch.object(
            remote_target,
            "build_release_bundle",
            return_value=(archive_path, metadata_path, {"commit": "deadbeef", "dirty_worktree": False}),
        ), patch.object(
            remote_target, "write_remote_update_script", return_value=update_script_path
        ), patch.object(
            remote_target, "copy_file_to_remote"
        ), patch.object(
            remote_target, "run_remote_update_install", return_value=0
        ), patch.object(
            remote_target, "run_remote_smoke", return_value=1
        ), patch.object(
            remote_target, "rollback_remote_update", return_value=0
        ) as rollback_remote_update:
            with self.assertRaises(SystemExit) as exc:
                remote_target.main(
                    [
                        "--env-file",
                        str(self.env_file),
                        "--receipts-dir",
                        str(self.receipts_dir),
                        "update",
                    ]
                )

        self.assertIn("rollback attempted", str(exc.exception).lower())
        rollback_remote_update.assert_called_once()
        receipt = json.loads(self.receipt_path.read_text(encoding="utf-8"))
        self.assertEqual(receipt["metadata"]["last_deployed_commit"], "abc123")

    def test_make_targets_dispatch_to_remote_helper(self) -> None:
        result = subprocess.run(
            [
                "make",
                "-n",
                f"ENV_LOCAL={self.env_file}",
                f"REMOTE_RECEIPTS_DIR={self.receipts_dir}",
                "REMOTE=blog-prod",
                "remote-use",
                "remote-status",
                "remote-logs",
                "remote-start",
                "remote-stop",
                "remote-update",
                "remote-open-dashboard",
            ],
            cwd=remote_target.REPO_ROOT,
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertIn('python3 ./scripts/manage_remote_target.py --env-file "', result.stdout)
        self.assertIn(' use --name "blog-prod"', result.stdout)
        self.assertIn("./scripts/manage_remote_target.py --env-file ", result.stdout)
        self.assertIn(" status --name ", result.stdout)
        self.assertIn(" logs --name ", result.stdout)
        self.assertIn(" start --name ", result.stdout)
        self.assertIn(" stop --name ", result.stdout)
        self.assertIn(" update --name ", result.stdout)
        self.assertIn(" open-dashboard --name ", result.stdout)


if __name__ == "__main__":
    unittest.main()
