import json
import subprocess
import tarfile
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "deploy" / "build_linode_release_bundle.py"


def run(
    repo_root: Path,
    archive_output: Path,
    metadata_output: Path,
) -> subprocess.CompletedProcess:
    return subprocess.run(
        [
            "python3",
            str(SCRIPT),
            "--repo-root",
            str(repo_root),
            "--archive-output",
            str(archive_output),
            "--metadata-output",
            str(metadata_output),
        ],
        cwd=str(REPO_ROOT),
        capture_output=True,
        text=True,
        check=False,
    )


class BuildLinodeReleaseBundleTests(unittest.TestCase):
    def create_git_repo(self) -> Path:
        temp_dir = Path(tempfile.mkdtemp(prefix="linode-bundle-test-"))
        subprocess.run(["git", "init"], cwd=temp_dir, check=True, capture_output=True)
        subprocess.run(
            ["git", "config", "user.name", "Test User"],
            cwd=temp_dir,
            check=True,
            capture_output=True,
        )
        subprocess.run(
            ["git", "config", "user.email", "test@example.com"],
            cwd=temp_dir,
            check=True,
            capture_output=True,
        )
        (temp_dir / "README.md").write_text("hello\n", encoding="utf-8")
        subprocess.run(["git", "add", "README.md"], cwd=temp_dir, check=True, capture_output=True)
        subprocess.run(
            ["git", "commit", "-m", "initial"],
            cwd=temp_dir,
            check=True,
            capture_output=True,
        )
        return temp_dir

    def test_builds_tarball_from_exact_head_commit_and_writes_metadata(self) -> None:
        repo_root = self.create_git_repo()
        archive_output = repo_root / "release.tar.gz"
        metadata_output = repo_root / "release.json"

        result = run(repo_root, archive_output, metadata_output)

        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        self.assertTrue(archive_output.exists())
        self.assertTrue(metadata_output.exists())

        metadata = json.loads(metadata_output.read_text(encoding="utf-8"))
        expected_head = (
            subprocess.run(
                ["git", "rev-parse", "HEAD"],
                cwd=repo_root,
                check=True,
                capture_output=True,
                text=True,
            )
            .stdout.strip()
        )
        self.assertEqual(metadata["commit"], expected_head)
        self.assertFalse(metadata["dirty_worktree"])

        with tarfile.open(archive_output, "r:gz") as archive:
            self.assertIn("README.md", archive.getnames())

    def test_reports_dirty_worktree_in_metadata(self) -> None:
        repo_root = self.create_git_repo()
        archive_output = repo_root / "release.tar.gz"
        metadata_output = repo_root / "release.json"
        (repo_root / "README.md").write_text("hello\nchanged\n", encoding="utf-8")

        result = run(repo_root, archive_output, metadata_output)

        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        metadata = json.loads(metadata_output.read_text(encoding="utf-8"))
        self.assertTrue(metadata["dirty_worktree"])

    def test_shuma_like_repo_builds_dashboard_assets_into_bundle(self) -> None:
        repo_root = self.create_git_repo()
        (repo_root / "Makefile").write_text(
            "dashboard-build:\n\t@mkdir -p dist/dashboard\n\t@printf '<h1>Dashboard</h1>\\n' > dist/dashboard/index.html\n",
            encoding="utf-8",
        )
        (repo_root / "spin.toml").write_text(
            '[component.dashboard-files]\nfiles = [{ source = "dist/dashboard", destination = "/" }]\n',
            encoding="utf-8",
        )
        (repo_root / "dashboard").mkdir()
        (repo_root / "package.json").write_text('{"private": true}\n', encoding="utf-8")
        node_modules_bin = repo_root / "node_modules" / ".bin"
        node_modules_bin.mkdir(parents=True)
        subprocess.run(
            ["git", "add", "Makefile", "spin.toml", "package.json", "dashboard"],
            cwd=repo_root,
            check=True,
            capture_output=True,
        )
        subprocess.run(
            ["git", "commit", "-m", "add dashboard build"],
            cwd=repo_root,
            check=True,
            capture_output=True,
        )

        archive_output = repo_root / "release.tar.gz"
        metadata_output = repo_root / "release.json"
        result = run(repo_root, archive_output, metadata_output)

        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        metadata = json.loads(metadata_output.read_text(encoding="utf-8"))
        self.assertTrue(metadata["dashboard_built"])

        with tarfile.open(archive_output, "r:gz") as archive:
            self.assertIn("dist/dashboard/index.html", archive.getnames())
            self.assertNotIn("node_modules", archive.getnames())


if __name__ == "__main__":
    unittest.main()
