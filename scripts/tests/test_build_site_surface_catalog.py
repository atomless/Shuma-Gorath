import json
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "build_site_surface_catalog.py"


def run_builder(docroot: Path, mode: str = "auto") -> subprocess.CompletedProcess:
    output_path = docroot / "surface-catalog.json"
    return subprocess.run(
        [
            "python3",
            str(SCRIPT),
            "--docroot",
            str(docroot),
            "--mode",
            mode,
            "--output",
            str(output_path),
        ],
        cwd=str(REPO_ROOT),
        capture_output=True,
        text=True,
        check=False,
    )


def load_output(docroot: Path) -> dict:
    return json.loads((docroot / "surface-catalog.json").read_text(encoding="utf-8"))


class BuildSiteSurfaceCatalogTests(unittest.TestCase):
    def test_builds_static_html_catalog_from_docroot(self) -> None:
        docroot = Path(tempfile.mkdtemp(prefix="docroot-static-"))
        (docroot / "index.html").write_text("<html>home</html>", encoding="utf-8")
        (docroot / "about.html").write_text("<html>about</html>", encoding="utf-8")
        (docroot / "blog").mkdir()
        (docroot / "blog" / "index.html").write_text("<html>blog</html>", encoding="utf-8")
        (docroot / "assets").mkdir()
        (docroot / "assets" / "app.js").write_text("console.log('ok')", encoding="utf-8")
        (docroot / ".git").mkdir()
        (docroot / ".git" / "HEAD").write_text("ref: refs/heads/main\n", encoding="utf-8")

        result = run_builder(docroot, mode="static-html-docroot")

        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        payload = load_output(docroot)
        inventory = {entry["path"]: entry for entry in payload["inventory"]}
        self.assertEqual(payload["mode"], "static-html-docroot")
        self.assertEqual(sorted(inventory), ["/", "/about.html", "/assets/app.js", "/blog/"])
        self.assertEqual(inventory["/"]["sources"], ["docroot:index"])
        self.assertEqual(inventory["/blog/"]["relative_file"], "blog/index.html")

    def test_merges_local_sitemap_entries_without_duplicate_paths(self) -> None:
        docroot = Path(tempfile.mkdtemp(prefix="docroot-sitemap-"))
        (docroot / "index.html").write_text("<html>home</html>", encoding="utf-8")
        (docroot / "sitemap.xml").write_text(
            textwrap.dedent(
                """\
                <?xml version="1.0" encoding="UTF-8"?>
                <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
                  <url><loc>https://example.com/</loc></url>
                  <url><loc>https://example.com/contact</loc></url>
                </urlset>
                """
            ),
            encoding="utf-8",
        )

        result = run_builder(docroot)

        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        payload = load_output(docroot)
        inventory = {entry["path"]: entry for entry in payload["inventory"]}
        self.assertEqual(sorted(inventory), ["/", "/contact", "/sitemap.xml"])
        self.assertEqual(inventory["/"]["sources"], ["docroot:index", "sitemap"])
        self.assertEqual(inventory["/contact"]["sources"], ["sitemap"])

    def test_maps_php_index_files_to_directory_routes(self) -> None:
        docroot = Path(tempfile.mkdtemp(prefix="docroot-php-"))
        (docroot / "index.php").write_text("<?php echo 'home';", encoding="utf-8")
        (docroot / "docs").mkdir()
        (docroot / "docs" / "index.php").write_text("<?php echo 'docs';", encoding="utf-8")
        (docroot / "feed.php").write_text("<?php echo 'feed';", encoding="utf-8")

        result = run_builder(docroot, mode="php-docroot")

        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)
        payload = load_output(docroot)
        inventory = {entry["path"]: entry for entry in payload["inventory"]}
        self.assertEqual(payload["mode"], "php-docroot")
        self.assertEqual(sorted(inventory), ["/", "/docs/", "/feed.php"])
        self.assertEqual(inventory["/docs/"]["relative_file"], "docs/index.php")


if __name__ == "__main__":
    unittest.main()
