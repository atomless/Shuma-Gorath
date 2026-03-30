import json
import subprocess
import tempfile
import textwrap
import time
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "build_sim_public_site.py"


class BuildSimPublicSiteTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = Path(tempfile.mkdtemp(prefix="sim-public-site-"))
        self.repo_root = self.temp_dir / "repo"
        self.repo_root.mkdir()
        self.artifact_root = self.temp_dir / "artifact"

        (self.repo_root / "README.md").write_text(
            "# About Shuma\n\nShuma protects sites from hostile automation.\n",
            encoding="utf-8",
        )
        (self.repo_root / "docs" / "research").mkdir(parents=True)
        (self.repo_root / "docs" / "plans").mkdir(parents=True)
        (self.repo_root / "todos").mkdir(parents=True)
        (self.repo_root / "config" / "sim_public_site").mkdir(parents=True)

        (self.repo_root / "docs" / "research" / "2026-03-30-alpha-research.md").write_text(
            textwrap.dedent(
                """\
                # Alpha Research

                This is the newest research note.

                - first signal
                - second signal
                """
            ),
            encoding="utf-8",
        )
        (self.repo_root / "docs" / "plans" / "2026-03-29-beta-plan.md").write_text(
            textwrap.dedent(
                """\
                # Beta Plan

                This plan follows the research note.

                ```rust
                fn main() {}
                ```
                """
            ),
            encoding="utf-8",
        )
        (self.repo_root / "todos" / "completed-todo-history.md").write_text(
            textwrap.dedent(
                """\
                # Completed TODO History

                ## Additional completions (2026-03-28)

                ### Shipped Something Important

                - [x] Completed the thing.
                - [x] Evidence:
                  - tests
                """
            ),
            encoding="utf-8",
        )
        (self.repo_root / "todos" / "todo.md").write_text(
            "# Active TODOs\n\n- [ ] Should stay out of the generated site.\n",
            encoding="utf-8",
        )
        (self.repo_root / "config" / "sim_public_site" / "corpus.toml").write_text(
            textwrap.dedent(
                """\
                [site]
                root_prefix = "/sim/public"
                about_source = "README.md"

                [sections.research]
                source_glob = "docs/research/2026-*.md"
                output_prefix = "research"

                [sections.plans]
                source_glob = "docs/plans/2026-*.md"
                output_prefix = "plans"

                [sections.work]
                source_file = "todos/completed-todo-history.md"
                output_prefix = "work"
                """
            ),
            encoding="utf-8",
        )

    def run_build(self) -> subprocess.CompletedProcess:
        command = [
            "python3",
            str(SCRIPT),
            "--repo-root",
            str(self.repo_root),
            "--artifact-root",
            str(self.artifact_root),
            "--corpus-config",
            str(self.repo_root / "config" / "sim_public_site" / "corpus.toml"),
            "--site-url",
            "https://example.test",
        ]
        return subprocess.run(
            command,
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            check=False,
        )

    def test_build_generates_semantic_feed_and_entry_pages(self) -> None:
        result = self.run_build()
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)

        site_root = self.artifact_root / "site"
        index_html = (site_root / "index.html").read_text(encoding="utf-8")
        about_html = (site_root / "about" / "index.html").read_text(encoding="utf-8")
        research_html = (
            site_root / "research" / "2026-03-30-alpha-research" / "index.html"
        ).read_text(encoding="utf-8")
        plan_html = (
            site_root / "plans" / "2026-03-29-beta-plan" / "index.html"
        ).read_text(encoding="utf-8")
        work_html = (
            site_root / "work" / "2026-03-28-shipped-something-important" / "index.html"
        ).read_text(encoding="utf-8")

        self.assertIn("<main>", index_html)
        self.assertIn("<article>", index_html)
        self.assertIn('href="/sim/public/about/"', index_html)
        self.assertIn('href="/sim/public/research/2026-03-30-alpha-research/"', index_html)
        self.assertIn('<time datetime="2026-03-30">', index_html)
        self.assertLess(index_html.index("Alpha Research"), index_html.index("Beta Plan"))
        self.assertLess(index_html.index("Beta Plan"), index_html.index("Shipped Something Important"))

        self.assertIn("<h1>About Shuma</h1>", about_html)
        self.assertNotIn("Active TODOs", index_html)
        self.assertIn("<h1>Alpha Research</h1>", research_html)
        self.assertIn("<h1>Beta Plan</h1>", plan_html)
        self.assertIn("<h1>Shipped Something Important</h1>", work_html)

        manifest = json.loads((self.artifact_root / "manifest.json").read_text(encoding="utf-8"))
        self.assertEqual(manifest["site_url"], "https://example.test")
        self.assertEqual(manifest["root_path"], "/sim/public/")
        self.assertEqual(manifest["about_path"], "/sim/public/about/")

        atom_xml = (site_root / "atom.xml").read_text(encoding="utf-8")
        self.assertIn("<feed", atom_xml)
        self.assertIn("https://example.test/sim/public/", atom_xml)
        self.assertIn("Alpha Research", atom_xml)

    def test_build_generates_discoverability_artifacts_and_pagination(self) -> None:
        for day in range(1, 25):
            (self.repo_root / "docs" / "research" / f"2026-02-{day:02d}-entry-{day}.md").write_text(
                f"# Research Entry {day}\n\nEntry {day} body.\n",
                encoding="utf-8",
            )

        result = self.run_build()
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)

        site_root = self.artifact_root / "site"
        index_html = (site_root / "index.html").read_text(encoding="utf-8")
        page_2_html = (site_root / "page" / "2" / "index.html").read_text(encoding="utf-8")
        research_index = (site_root / "research" / "index.html").read_text(encoding="utf-8")
        research_page_2 = (
            site_root / "research" / "page" / "2" / "index.html"
        ).read_text(encoding="utf-8")
        robots_txt = (site_root / "robots.txt").read_text(encoding="utf-8")
        sitemap_index = (site_root / "sitemap.xml").read_text(encoding="utf-8")
        pages_sitemap = (site_root / "sitemaps" / "pages.xml").read_text(encoding="utf-8")
        entries_sitemap = (site_root / "sitemaps" / "entries.xml").read_text(encoding="utf-8")

        self.assertIn('href="/sim/public/page/2/"', index_html)
        self.assertIn('rel="next"', index_html)
        self.assertIn("Research Entry 4", page_2_html)
        self.assertIn('href="/sim/public/research/page/2/"', research_index)
        self.assertIn('rel="prev"', research_page_2)
        self.assertIn("Sitemap: https://example.test/sim/public/sitemap.xml", robots_txt)
        self.assertIn("sitemaps/pages.xml", sitemap_index)
        self.assertIn("sitemaps/entries.xml", sitemap_index)
        self.assertIn("https://example.test/sim/public/about/", pages_sitemap)
        self.assertIn(
            "https://example.test/sim/public/research/2026-03-30-alpha-research/",
            entries_sitemap,
        )

    def test_if_stale_hours_skips_fresh_rebuilds(self) -> None:
        first = self.run_build()
        self.assertEqual(first.returncode, 0, msg=first.stderr or first.stdout)

        freshness_path = self.artifact_root / "freshness.json"
        first_freshness = freshness_path.read_text(encoding="utf-8")
        time.sleep(1.1)
        second = subprocess.run(
            [
                "python3",
                str(SCRIPT),
                "--repo-root",
                str(self.repo_root),
                "--artifact-root",
                str(self.artifact_root),
                "--corpus-config",
                str(self.repo_root / "config" / "sim_public_site" / "corpus.toml"),
                "--site-url",
                "https://example.test",
                "--if-stale-hours",
                "24",
            ],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(second.returncode, 0, msg=second.stderr or second.stdout)
        self.assertEqual(freshness_path.read_text(encoding="utf-8"), first_freshness)

    def test_build_uses_commonmark_renderer_for_markdown_blocks(self) -> None:
        result = self.run_build()
        self.assertEqual(result.returncode, 0, msg=result.stderr or result.stdout)

        research_html = (
            self.artifact_root
            / "site"
            / "research"
            / "2026-03-30-alpha-research"
            / "index.html"
        ).read_text(encoding="utf-8")
        plan_html = (
            self.artifact_root
            / "site"
            / "plans"
            / "2026-03-29-beta-plan"
            / "index.html"
        ).read_text(encoding="utf-8")

        self.assertIn("<ul>", research_html)
        self.assertIn("<li>first signal</li>", research_html)
        self.assertIn("<pre><code class=\"language-rust\">", plan_html)


if __name__ == "__main__":
    unittest.main()
