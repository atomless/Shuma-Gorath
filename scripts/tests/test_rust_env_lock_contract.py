import re
import unittest
from pathlib import Path
from typing import Iterator, List, Optional, Tuple


REPO_ROOT = Path(__file__).resolve().parents[2]
MAKEFILE = REPO_ROOT / "Makefile"
RUST_FILE_GLOBS = ("src/**/*.rs", "tests/**/*.rs")
ENV_MUTATION_MARKERS = (
    "std::env::set_var(",
    "std::env::remove_var(",
    "env::set_var(",
    "env::remove_var(",
    "clear_env(",
    "clear_gateway_env(",
    "set_gateway_env_baseline(",
)
ENV_LOCK_GUARDS = (
    "crate::test_support::lock_env(",
    "lock_env(",
    "with_runtime_env(",
    "with_runtime_env_overrides(",
)
TEST_FUNCTION_RE = re.compile(
    r"(?ms)(?P<attrs>(?:\s*#\[[^\]]+\]\s*)+)"
    r"(?:(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?)?"
    r"fn\s+(?P<name>[A-Za-z0-9_]+)\s*\([^)]*\)\s*\{"
)


def extract_target_body(target: str, source: str) -> Optional[str]:
    match = re.search(
        rf"^{re.escape(target)}:.*?(?=^[A-Za-z0-9_.-]+:|\Z)",
        source,
        re.MULTILINE | re.DOTALL,
    )
    if not match:
        return None
    return match.group(0)


def iter_rust_test_functions(path: Path) -> Iterator[Tuple[str, str]]:
    source = path.read_text(encoding="utf-8")
    for match in TEST_FUNCTION_RE.finditer(source):
        attrs = match.group("attrs")
        if "#[test]" not in attrs and "tokio::test" not in attrs:
            continue

        start = match.end()
        depth = 1
        cursor = start
        while cursor < len(source) and depth:
            char = source[cursor]
            if char == "{":
                depth += 1
            elif char == "}":
                depth -= 1
            cursor += 1

        yield match.group("name"), source[start : cursor - 1]


def rust_test_env_lock_offenders() -> List[str]:
    offenders: List[str] = []
    for pattern in RUST_FILE_GLOBS:
        for path in sorted(REPO_ROOT.glob(pattern)):
            for name, body in iter_rust_test_functions(path):
                if not any(marker in body for marker in ENV_MUTATION_MARKERS):
                    continue
                if any(guard in body for guard in ENV_LOCK_GUARDS):
                    continue
                offenders.append(f"{path.relative_to(REPO_ROOT)}::{name}")
    return offenders


class RustEnvLockContractTests(unittest.TestCase):
    def test_makefile_exposes_explicit_rust_env_lock_contract_target(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        body = extract_target_body("test-rust-env-lock-contract", source)
        self.assertIsNotNone(body)
        self.assertIn("scripts/tests/test_rust_env_lock_contract.py", body)

    def test_canonical_test_suite_runs_rust_env_lock_contract(self) -> None:
        source = MAKEFILE.read_text(encoding="utf-8")
        body = extract_target_body("test", source)
        self.assertIsNotNone(body)
        self.assertIn("test-rust-env-lock-contract", body)

    def test_all_rust_env_mutating_tests_hold_env_lock_or_wrapper(self) -> None:
        offenders = rust_test_env_lock_offenders()
        self.assertEqual(
            offenders,
            [],
            msg=(
                "Rust tests mutating process env must hold lock_env() or use an approved "
                f"wrapper. Offenders: {offenders}"
            ),
        )


if __name__ == "__main__":
    unittest.main()
