import re
import unittest
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
RUST_ROOT = REPO_ROOT / "src"
TEST_ATTR_RE = re.compile(r"^\s*#\[(?:[A-Za-z0-9_:]+::)?test(?:\([^\]]*\))?\]\s*$")
FN_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z0-9_]+)")
ENV_MUTATION_RE = re.compile(r"std::env::(?:set_var|remove_var)\s*\(")
LOCK_RE = re.compile(r"lock_env\s*\(")


@dataclass
class TestFunction:
    name: str
    body: str
    start_line: int


def extract_test_functions(source: str) -> list[TestFunction]:
    lines = source.splitlines()
    functions: list[TestFunction] = []
    pending_test = False
    index = 0

    while index < len(lines):
        line = lines[index]
        stripped = line.strip()

        if TEST_ATTR_RE.match(line):
            pending_test = True
            index += 1
            continue

        if pending_test and stripped.startswith("#["):
            index += 1
            continue

        fn_match = FN_RE.match(line)
        if pending_test and fn_match:
            body_lines = [line]
            brace_depth = line.count("{") - line.count("}")
            index += 1

            while index < len(lines):
                body_line = lines[index]
                body_lines.append(body_line)
                brace_depth += body_line.count("{") - body_line.count("}")
                index += 1
                if brace_depth <= 0:
                    break

            functions.append(
                TestFunction(
                    name=fn_match.group(1),
                    body="\n".join(body_lines),
                    start_line=index - len(body_lines) + 1,
                )
            )
            pending_test = False
            continue

        pending_test = False
        index += 1

    return functions


def find_env_isolation_offenders(path: Path) -> list[tuple[str, int]]:
    source = path.read_text(encoding="utf-8")
    offenders: list[tuple[str, int]] = []

    for function in extract_test_functions(source):
        mutation = ENV_MUTATION_RE.search(function.body)
        if mutation is None:
            continue
        lock = LOCK_RE.search(function.body)
        if lock is None or lock.start() > mutation.start():
            offenders.append((function.name, function.start_line))

    return offenders


class RustEnvIsolationContractTests(unittest.TestCase):
    def test_parser_flags_env_mutation_without_lock(self) -> None:
        source = """
#[test]
fn sample() {
    std::env::set_var("SHUMA_FOO", "bar");
}
"""

        offenders = []
        for function in extract_test_functions(source):
            mutation = ENV_MUTATION_RE.search(function.body)
            lock = LOCK_RE.search(function.body)
            if mutation and (lock is None or lock.start() > mutation.start()):
                offenders.append(function.name)

        self.assertEqual(offenders, ["sample"])

    def test_parser_accepts_lock_before_env_mutation(self) -> None:
        source = """
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn sample() {
    let _lock = crate::test_support::lock_env();
    std::env::set_var("SHUMA_FOO", "bar");
}
"""

        offenders = []
        for function in extract_test_functions(source):
            mutation = ENV_MUTATION_RE.search(function.body)
            lock = LOCK_RE.search(function.body)
            if mutation and (lock is None or lock.start() > mutation.start()):
                offenders.append(function.name)

        self.assertEqual(offenders, [])

    def test_repo_rust_tests_hold_lock_before_env_mutation(self) -> None:
        repo_offenders: list[str] = []

        for path in sorted(RUST_ROOT.rglob("*.rs")):
            offenders = find_env_isolation_offenders(path)
            for name, line in offenders:
                repo_offenders.append(f"{path.relative_to(REPO_ROOT)}:{line} {name}")

        self.assertEqual(
            repo_offenders,
            [],
            msg=(
                "Rust tests that mutate process env must acquire lock_env() before the first "
                "mutation. Offenders:\n- " + "\n- ".join(repo_offenders)
            ),
        )


if __name__ == "__main__":
    unittest.main()
