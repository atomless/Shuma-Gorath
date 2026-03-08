#!/usr/bin/env python3

import subprocess
import unittest
from unittest.mock import patch

import scripts.tests.playwright_runtime as playwright_runtime


class PlaywrightRuntimeUnitTests(unittest.TestCase):
    def test_ensure_playwright_chromium_installs_missing_browser_in_repo_cache(self):
        cache_path = str(playwright_runtime.DEFAULT_PLAYWRIGHT_BROWSER_CACHE)
        chromium_path = f"{cache_path}/chromium/chrome"

        with patch("scripts.tests.playwright_runtime.subprocess.run") as run_mock, patch(
            "scripts.tests.playwright_runtime.os.access", return_value=True
        ):
            run_mock.side_effect = [
                subprocess.CompletedProcess(
                    args=["corepack", "pnpm", "exec", "node", "-e"],
                    returncode=0,
                    stdout="",
                    stderr="",
                ),
                subprocess.CompletedProcess(
                    args=["corepack", "pnpm", "exec", "playwright", "install", "chromium"],
                    returncode=0,
                    stdout="installed",
                    stderr="",
                ),
                subprocess.CompletedProcess(
                    args=["corepack", "pnpm", "exec", "node", "-e"],
                    returncode=0,
                    stdout=f"{chromium_path}\n",
                    stderr="",
                ),
            ]
            status = playwright_runtime.ensure_playwright_chromium()

        self.assertEqual(status.browser_cache, cache_path)
        self.assertEqual(status.chromium_executable, chromium_path)
        self.assertTrue(status.installed_now)
        self.assertEqual(run_mock.call_count, 3)
        for call in run_mock.call_args_list:
            env = call.kwargs.get("env") or {}
            self.assertEqual(env.get("PLAYWRIGHT_BROWSERS_PATH"), cache_path)


if __name__ == "__main__":
    unittest.main()
