import json
import re
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
        self.receipts_dir = self.temp_dir / ".shuma" / "remotes"
        self.receipts_dir.mkdir(parents=True, exist_ok=True)
        self.receipt_path = self.receipts_dir / "blog-prod.json"
        (self.temp_dir / "catalog.json").write_text('{"inventory":[{"path":"/"}]}\n', encoding="utf-8")
        self.scrapling_scope_path = self.temp_dir / "scrapling.scope.json"
        self.scrapling_scope_path.write_text("{}\n", encoding="utf-8")
        self.scrapling_seed_path = self.temp_dir / "scrapling.seed.json"
        self.scrapling_seed_path.write_text("{}\n", encoding="utf-8")
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
                    "upstream_origin": "http://127.0.0.1:8080",
                    "scrapling": {
                        "scope_descriptor_path": str(self.scrapling_scope_path),
                        "seed_inventory_path": str(self.scrapling_seed_path),
                        "remote_scope_descriptor_path": "/opt/shuma-gorath/.shuma/adversary-sim/scrapling-scope.json",
                        "remote_seed_inventory_path": "/opt/shuma-gorath/.shuma/adversary-sim/scrapling-seed-inventory.json",
                        "remote_crawldir_path": "/opt/shuma-gorath/.shuma/adversary-sim/scrapling-crawldir",
                    },
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

    def test_default_remote_receipts_dir_uses_durable_local_state_dir_not_spin(self) -> None:
        self.assertEqual(remote_target.DEFAULT_REMOTE_RECEIPTS_DIR, remote_target.REPO_ROOT / ".shuma" / "remotes")
        self.assertNotIn("/.spin/", str(remote_target.DEFAULT_REMOTE_RECEIPTS_DIR))

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
            remote_target, "run_remote_loopback_health_check", return_value=0
        ) as run_loopback_health, patch.object(
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
                "/tmp/shuma-remote-update-scrapling-scope.json",
                "/tmp/shuma-remote-update-scrapling-seed.json",
                "/tmp/shuma-remote-update.sh",
            ],
        )
        run_install.assert_called_once()
        run_loopback_health.assert_called_once()
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
            remote_target, "run_remote_loopback_health_check", return_value=0
        ) as run_install, patch.object(
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

    def test_update_uploads_scrapling_artifacts_when_receipt_declares_them(self) -> None:
        self.env_file.write_text("SHUMA_ACTIVE_REMOTE=blog-prod\n", encoding="utf-8")
        receipt = json.loads(self.receipt_path.read_text(encoding="utf-8"))
        local_scope = self.temp_dir / "scrapling.scope.json"
        local_seed = self.temp_dir / "scrapling.seed.json"
        local_scope.write_text('{"allowed_hosts":["blog.example.com"]}\n', encoding="utf-8")
        local_seed.write_text('{"inventory":[{"url":"https://blog.example.com/"}]}\n', encoding="utf-8")
        receipt["deploy"]["scrapling"] = {
            "scope_descriptor_path": str(local_scope),
            "seed_inventory_path": str(local_seed),
            "remote_scope_descriptor_path": "/opt/shuma-gorath/.shuma/adversary-sim/scrapling-scope.json",
            "remote_seed_inventory_path": "/opt/shuma-gorath/.shuma/adversary-sim/scrapling-seed-inventory.json",
            "remote_crawldir_path": "/opt/shuma-gorath/.shuma/adversary-sim/scrapling-crawldir",
        }
        self.receipt_path.write_text(json.dumps(receipt) + "\n", encoding="utf-8")

        bundle_dir = self.temp_dir / "bundle-scrapling"
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
        ), patch.object(
            remote_target, "run_remote_loopback_health_check", return_value=0
        ), patch.object(
            remote_target, "run_remote_smoke", return_value=0
        ):
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
                "/tmp/shuma-remote-update-scrapling-scope.json",
                "/tmp/shuma-remote-update-scrapling-seed.json",
                "/tmp/shuma-remote-update.sh",
            ],
        )

    def test_run_remote_smoke_uses_allowlisted_forwarded_ip_for_public_remote(self) -> None:
        self.env_file.write_text(
            "SHUMA_ADMIN_IP_ALLOWLIST=198.51.100.10/32\nSHUMA_GATEWAY_UPSTREAM_ORIGIN=https://foreign.example.com\n",
            encoding="utf-8",
        )
        receipt = remote_target.load_remote_receipt(self.receipts_dir, "blog-prod")

        with patch.object(subprocess, "run") as run:
            run.return_value = subprocess.CompletedProcess(args=[], returncode=0)
            rc = remote_target.run_remote_smoke(self.env_file, receipt)

        self.assertEqual(rc, 0)
        env = run.call_args.kwargs["env"]
        self.assertEqual(env["SHUMA_BASE_URL"], "https://blog.example.com")
        self.assertEqual(env["SHUMA_SMOKE_SKIP_HEALTH"], "1")
        self.assertEqual(env["SHUMA_GATEWAY_UPSTREAM_ORIGIN"], "http://127.0.0.1:8080")
        self.assertEqual(env["SHUMA_SMOKE_FORWARDED_IP"], "198.51.100.10")
        self.assertEqual(env["SHUMA_SMOKE_ADMIN_FORWARDED_IP"], "198.51.100.10")
        self.assertEqual(env["GATEWAY_SURFACE_CATALOG_PATH"], str((self.temp_dir / "catalog.json").resolve()))

    def test_run_remote_smoke_relaxes_tls_only_for_sslip_proof_domains(self) -> None:
        self.env_file.write_text("SHUMA_ADMIN_IP_ALLOWLIST=198.51.100.10/32\n", encoding="utf-8")
        receipt = remote_target.load_remote_receipt(self.receipts_dir, "blog-prod")
        receipt["runtime"]["public_base_url"] = "https://172.239.98.201.sslip.io"

        with patch.object(subprocess, "run") as run:
            run.return_value = subprocess.CompletedProcess(args=[], returncode=0)
            rc = remote_target.run_remote_smoke(self.env_file, receipt)

        self.assertEqual(rc, 0)
        remote_command = run.call_args.args[0][-1]
        self.assertIn("smoke_single_host.sh", remote_command)
        self.assertIn("SHUMA_BASE_URL=https://172.239.98.201.sslip.io", remote_command)
        self.assertIn("SHUMA_GATEWAY_UPSTREAM_ORIGIN=http://127.0.0.1:8080", remote_command)
        self.assertIn("SHUMA_SMOKE_INSECURE_TLS=true", remote_command)
        self.assertIn("SHUMA_SMOKE_SKIP_RESERVED_ROUTES=true", remote_command)
        self.assertIn(
            "GATEWAY_SURFACE_CATALOG_PATH=/tmp/shuma-remote-update-surface-catalog.json",
            remote_command,
        )

    def test_run_remote_smoke_hydrates_missing_secrets_from_remote_env(self) -> None:
        self.env_file.write_text("SHUMA_ADMIN_IP_ALLOWLIST=198.51.100.10/32\n", encoding="utf-8")
        receipt = remote_target.load_remote_receipt(self.receipts_dir, "blog-prod")

        def fake_run(*args, **kwargs):
            command = args[0]
            if command[0] == "ssh":
                return subprocess.CompletedProcess(
                    args=command,
                    returncode=0,
                    stdout="\n".join(
                        [
                            "SHUMA_API_KEY=remote-api-key",
                            "SHUMA_FORWARDED_IP_SECRET=remote-forward-secret",
                            "SHUMA_HEALTH_SECRET=remote-health-secret",
                        ]
                    )
                    + "\n",
                    stderr="",
                )
            return subprocess.CompletedProcess(args=command, returncode=0)

        with patch.object(subprocess, "run", side_effect=fake_run) as run:
            rc = remote_target.run_remote_smoke(self.env_file, receipt)

        self.assertEqual(rc, 0)
        env = run.call_args.kwargs["env"]
        self.assertEqual(env["SHUMA_API_KEY"], "remote-api-key")
        self.assertEqual(env["SHUMA_FORWARDED_IP_SECRET"], "remote-forward-secret")
        self.assertEqual(env["SHUMA_HEALTH_SECRET"], "remote-health-secret")
        env_text = self.env_file.read_text(encoding="utf-8")
        self.assertIn("SHUMA_API_KEY=remote-api-key", env_text)
        self.assertIn("SHUMA_FORWARDED_IP_SECRET=remote-forward-secret", env_text)
        self.assertIn("SHUMA_HEALTH_SECRET=remote-health-secret", env_text)

    def test_run_remote_loopback_health_check_waits_for_slow_spin_startup(self) -> None:
        receipt = remote_target.load_remote_receipt(self.receipts_dir, "blog-prod")
        attempts_before_ready = 7
        results = [
            subprocess.CompletedProcess(args=["ssh"], returncode=1, stdout="", stderr="status=000 body=")
            for _ in range(attempts_before_ready - 1)
        ]
        results.append(subprocess.CompletedProcess(args=["ssh"], returncode=0, stdout="", stderr=""))

        with patch.object(subprocess, "run", side_effect=results) as run, patch.object(
            remote_target.time, "sleep"
        ) as sleep:
            rc = remote_target.run_remote_loopback_health_check(receipt)

        self.assertEqual(rc, 0)
        self.assertEqual(run.call_count, attempts_before_ready)
        self.assertEqual(sleep.call_count, attempts_before_ready - 1)
        sleep.assert_called_with(remote_target.REMOTE_LOOPBACK_HEALTH_RETRY_DELAY_SECONDS)

    def test_run_remote_loopback_health_check_returns_last_failure_after_budget_exhausted(self) -> None:
        receipt = remote_target.load_remote_receipt(self.receipts_dir, "blog-prod")
        failure = subprocess.CompletedProcess(args=["ssh"], returncode=1, stdout="", stderr="status=000 body=")

        with patch.object(
            subprocess, "run", side_effect=[failure] * remote_target.REMOTE_LOOPBACK_HEALTH_ATTEMPTS
        ) as run, patch.object(remote_target.time, "sleep") as sleep:
            rc = remote_target.run_remote_loopback_health_check(receipt)

        self.assertEqual(rc, 1)
        self.assertEqual(run.call_count, remote_target.REMOTE_LOOPBACK_HEALTH_ATTEMPTS)
        self.assertEqual(sleep.call_count, remote_target.REMOTE_LOOPBACK_HEALTH_ATTEMPTS - 1)

    def test_update_builds_uploads_restarts_runs_loopback_health_smokes_and_refreshes_receipt_metadata(self) -> None:
        self.env_file.write_text("SHUMA_ACTIVE_REMOTE=blog-prod\n", encoding="utf-8")
        bundle_dir = self.temp_dir / "bundle-update"
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
            remote_target, "run_remote_loopback_health_check", return_value=0
        ) as run_loopback_health, patch.object(
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
                "/tmp/shuma-remote-update-scrapling-scope.json",
                "/tmp/shuma-remote-update-scrapling-seed.json",
                "/tmp/shuma-remote-update.sh",
            ],
        )
        run_install.assert_called_once()
        run_loopback_health.assert_called_once()
        run_smoke.assert_called_once()
        receipt = json.loads(self.receipt_path.read_text(encoding="utf-8"))
        self.assertEqual(receipt["metadata"]["last_deployed_commit"], "deadbeef")
        self.assertTrue(receipt["metadata"]["last_deployed_at_utc"].endswith("Z"))

    def test_update_backfills_missing_scrapling_metadata_from_canonical_deploy_prep(self) -> None:
        self.env_file.write_text("SHUMA_ACTIVE_REMOTE=blog-prod\n", encoding="utf-8")
        payload = json.loads(self.receipt_path.read_text(encoding="utf-8"))
        payload["deploy"].pop("scrapling", None)
        self.receipt_path.write_text(json.dumps(payload) + "\n", encoding="utf-8")
        prepared_scope_path = self.temp_dir / "prepared-scrapling.scope.json"
        prepared_scope_path.write_text("{}\n", encoding="utf-8")
        prepared_seed_path = self.temp_dir / "prepared-scrapling.seed.json"
        prepared_seed_path.write_text("{}\n", encoding="utf-8")
        prepared_receipt = {
            "artifacts": {
                "scope_descriptor_path": str(prepared_scope_path),
                "seed_inventory_path": str(prepared_seed_path),
            },
            "environment": {
                "remote": {
                    "ADVERSARY_SIM_SCRAPLING_SCOPE_DESCRIPTOR_PATH": "/opt/shuma-gorath/.shuma/adversary-sim/scrapling-scope.json",
                    "ADVERSARY_SIM_SCRAPLING_SEED_INVENTORY_PATH": "/opt/shuma-gorath/.shuma/adversary-sim/scrapling-seed-inventory.json",
                    "ADVERSARY_SIM_SCRAPLING_CRAWLDIR": "/opt/shuma-gorath/.shuma/adversary-sim/scrapling-crawldir",
                }
            },
        }
        bundle_dir = self.temp_dir / "bundle-update-backfill"
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
        ), patch(
            "scripts.deploy.scrapling_deploy_prep.prepare_scrapling_deploy",
            return_value=prepared_receipt,
        ) as prepare_scrapling_deploy, patch.object(
            remote_target, "write_remote_update_script", return_value=update_script_path
        ), patch.object(
            remote_target, "copy_file_to_remote"
        ) as copy_file, patch.object(
            remote_target, "run_remote_update_install", return_value=0
        ), patch.object(
            remote_target, "run_remote_loopback_health_check", return_value=0
        ), patch.object(
            remote_target, "run_remote_smoke", return_value=0
        ):
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
        prepare_scrapling_deploy.assert_called_once_with(
            public_base_url="https://blog.example.com",
            runtime_mode="ssh_systemd",
        )
        copied_remote_paths = [call.args[2] for call in copy_file.call_args_list]
        self.assertEqual(
            copied_remote_paths,
            [
                "/tmp/shuma-remote-update-release.tar.gz",
                "/tmp/shuma-remote-update-release.json",
                "/tmp/shuma-remote-update-surface-catalog.json",
                "/tmp/shuma-remote-update-scrapling-scope.json",
                "/tmp/shuma-remote-update-scrapling-seed.json",
                "/tmp/shuma-remote-update.sh",
            ],
        )
        receipt = json.loads(self.receipt_path.read_text(encoding="utf-8"))
        self.assertEqual(
            receipt["deploy"]["scrapling"],
            {
                "scope_descriptor_path": str(prepared_scope_path),
                "seed_inventory_path": str(prepared_seed_path),
                "remote_scope_descriptor_path": "/opt/shuma-gorath/.shuma/adversary-sim/scrapling-scope.json",
                "remote_seed_inventory_path": "/opt/shuma-gorath/.shuma/adversary-sim/scrapling-seed-inventory.json",
                "remote_crawldir_path": "/opt/shuma-gorath/.shuma/adversary-sim/scrapling-crawldir",
            },
        )

    def test_update_attempts_rollback_when_remote_loopback_health_fails(self) -> None:
        self.env_file.write_text("SHUMA_ACTIVE_REMOTE=blog-prod\n", encoding="utf-8")
        bundle_dir = self.temp_dir / "bundle-loopback-fail"
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
            remote_target, "run_remote_loopback_health_check", return_value=1
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

        self.assertIn("loopback health failed", str(exc.exception).lower())
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

    def test_remote_update_script_seeds_latest_env_defaults_before_restoring_remote_overlay(self) -> None:
        work_dir = self.temp_dir / "script-work"
        work_dir.mkdir()

        script_path = remote_target.write_remote_update_script(work_dir)
        script = script_path.read_text(encoding="utf-8")

        self.assertIn('make setup-runtime', script)
        self.assertIn('cp "${REMOTE_APP_DIR}/.env.local" "${PREV_ENV_OVERLAY_PATH}"', script)
        self.assertIn(
            'python3 scripts/deploy/merge_env_overlay.py --overlay "${PREV_ENV_OVERLAY_PATH}" --env-file ".env.local" --set "SHUMA_GATEWAY_UPSTREAM_ORIGIN=${REMOTE_UPSTREAM_ORIGIN}" --set "SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL=${REMOTE_ALLOW_INSECURE_HTTP_LOCAL}" --set "SHUMA_SPIN_MANIFEST=${REMOTE_APP_DIR}/spin.gateway.toml"',
            script,
        )
        self.assertIn('export SHUMA_GATEWAY_UPSTREAM_ORIGIN="${REMOTE_UPSTREAM_ORIGIN}"', script)
        self.assertIn('export SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL="${REMOTE_ALLOW_INSECURE_HTTP_LOCAL}"', script)
        self.assertIn('export SHUMA_SPIN_MANIFEST="${NEXT_APP_DIR}/spin.gateway.toml"', script)
        self.assertIn(
            'GATEWAY_SURFACE_CATALOG_PATH="${GATEWAY_SURFACE_CATALOG_REMOTE_PATH}"',
            script,
        )
        self.assertIn(
            'SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL="${REMOTE_ALLOW_INSECURE_HTTP_LOCAL}"',
            script,
        )
        self.assertIn(
            'SHUMA_SPIN_MANIFEST="${NEXT_APP_DIR}/spin.gateway.toml"',
            script,
        )
        self.assertIn(
            'SHUMA_GATEWAY_UPSTREAM_ORIGIN="${REMOTE_UPSTREAM_ORIGIN}"',
            script,
        )
        self.assertIn(
            'deploy-self-hosted-minimal-prebuilt',
            script,
        )
        self.assertIsNone(
            re.search(
                r'GATEWAY_SURFACE_CATALOG_PATH="\$\{GATEWAY_SURFACE_CATALOG_REMOTE_PATH\}" make deploy-self-hosted-minimal(?:\s|$)',
                script,
            )
        )
        self.assertNotIn('cp "${REMOTE_APP_DIR}/.env.local" "${NEXT_APP_DIR}/.env.local"', script)

    def test_remote_update_script_wires_scrapling_scope_seed_and_env(self) -> None:
        work_dir = self.temp_dir / "script-work-scrapling"
        work_dir.mkdir()

        script_path = remote_target.write_remote_update_script(work_dir)
        script = script_path.read_text(encoding="utf-8")

        self.assertIn(': "${SCRAPLING_SCOPE_REMOTE_SOURCE_PATH:=}"', script)
        self.assertIn(': "${SCRAPLING_SEED_REMOTE_SOURCE_PATH:=}"', script)
        self.assertIn(': "${SCRAPLING_SCOPE_REMOTE_DEST_PATH:=}"', script)
        self.assertIn(': "${SCRAPLING_SEED_REMOTE_DEST_PATH:=}"', script)
        self.assertIn(': "${SCRAPLING_CRAWLDIR_REMOTE_DEST_PATH:=}"', script)
        self.assertIn('mkdir -p "$(dirname "${SCRAPLING_SCOPE_REMOTE_DEST_PATH}")"', script)
        self.assertIn('cp "${SCRAPLING_SCOPE_REMOTE_SOURCE_PATH}" "${SCRAPLING_SCOPE_REMOTE_DEST_PATH}"', script)
        self.assertIn('cp "${SCRAPLING_SEED_REMOTE_SOURCE_PATH}" "${SCRAPLING_SEED_REMOTE_DEST_PATH}"', script)
        self.assertIn('mkdir -p "${SCRAPLING_CRAWLDIR_REMOTE_DEST_PATH}"', script)
        self.assertIn(
            '--set "ADVERSARY_SIM_SCRAPLING_SCOPE_DESCRIPTOR_PATH=${SCRAPLING_SCOPE_REMOTE_DEST_PATH}"',
            script,
        )
        self.assertIn(
            '--set "ADVERSARY_SIM_SCRAPLING_SEED_INVENTORY_PATH=${SCRAPLING_SEED_REMOTE_DEST_PATH}"',
            script,
        )
        self.assertIn(
            '--set "ADVERSARY_SIM_SCRAPLING_CRAWLDIR=${SCRAPLING_CRAWLDIR_REMOTE_DEST_PATH}"',
            script,
        )

    def test_load_remote_receipt_backfills_default_linode_upstream_origin_for_old_receipts(self) -> None:
        payload = json.loads(self.receipt_path.read_text(encoding="utf-8"))
        payload["deploy"].pop("upstream_origin", None)
        self.receipt_path.write_text(json.dumps(payload) + "\n", encoding="utf-8")

        receipt = remote_target.load_remote_receipt(self.receipts_dir, "blog-prod")

        self.assertEqual(receipt["deploy"]["upstream_origin"], remote_target.DEFAULT_SHARED_HOST_UPSTREAM_ORIGIN)


if __name__ == "__main__":
    unittest.main()
