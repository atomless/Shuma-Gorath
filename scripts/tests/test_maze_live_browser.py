#!/usr/bin/env python3

import unittest

import scripts.tests.maze_live_browser as maze_live_browser
from scripts.tests.maze_live_traversal import MazeTraversalFailure


class MazeLiveBrowserUnitTests(unittest.TestCase):
    def test_build_opaque_entry_path_preserves_prefix(self):
        path = maze_live_browser.build_opaque_entry_path("/_/abcdef123456/", "Maze High Confidence", 3)
        self.assertEqual(path, "/_/abcdef123456/maze-high-confidence-3")

    def test_build_opaque_entry_path_rejects_invalid_prefix(self):
        with self.assertRaises(MazeTraversalFailure):
            maze_live_browser.build_opaque_entry_path("/maze/", "bad", 1)

    def test_browser_request_path_seen_matches_exact_path(self):
        evidence = {
            "request_lineage": [
                {"path": "/_/abcdef123456/checkpoint"},
                {"path": "/_/abcdef123456/issue-links"},
            ]
        }
        self.assertTrue(
            maze_live_browser.browser_request_path_seen(
                evidence, "/_/abcdef123456/checkpoint"
            )
        )
        self.assertFalse(
            maze_live_browser.browser_request_path_seen(
                evidence, "/_/abcdef123456/missing"
            )
        )


if __name__ == "__main__":
    unittest.main()
