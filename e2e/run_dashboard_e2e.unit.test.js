const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { execFileSync } = require('node:child_process');

const ROOT_DIR = path.resolve(__dirname, '..');
const RUNNER_PATH = path.join(ROOT_DIR, 'scripts', 'tests', 'run_dashboard_e2e.sh');
const REPO_CACHE_PATH = path.join(ROOT_DIR, '.cache', 'ms-playwright');
const REPO_LOCAL_HOME_PATH = path.join(ROOT_DIR, '.cache', 'playwright-home');

function escapeRegExp(value) {
  return String(value).replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function writeExecutable(filePath, contents) {
  fs.writeFileSync(filePath, contents, { mode: 0o755 });
}

test('runner falls back to system browser cache after sandbox preflight failures', () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'run-dashboard-e2e-unit-'));
  const fakeBinDir = path.join(tempDir, 'bin');
  const systemHome = path.join(tempDir, 'system-home');
  const traceFile = path.join(tempDir, 'trace.log');
  fs.mkdirSync(fakeBinDir, { recursive: true });
  fs.mkdirSync(systemHome, { recursive: true });

  const fakeCorepackPath = path.join(fakeBinDir, 'corepack');
  writeExecutable(
    fakeCorepackPath,
    `#!/bin/bash
set -euo pipefail
trace_file="\${FAKE_COREPACK_TRACE_FILE:?}"

if [[ "$1" != "pnpm" ]]; then
  echo "unexpected corepack args: $*" >> "$trace_file"
  exit 99
fi
shift

if [[ "$1" == "exec" && "$2" == "node" && "$3" == "-e" ]]; then
  if [[ -n "\${PLAYWRIGHT_BROWSERS_PATH:-}" ]]; then
    echo "\${PLAYWRIGHT_BROWSERS_PATH}/chromium-not-installed"
  else
    echo "\${HOME}/Library/Caches/ms-playwright/chromium-not-installed"
  fi
  exit 0
fi

if [[ "$1" == "exec" && "$2" == "playwright" && "$3" == "install" && "$4" == "chromium" ]]; then
  echo "install HOME=\${HOME} PWB=\${PLAYWRIGHT_BROWSERS_PATH:-<unset>}" >> "$trace_file"
  exit 0
fi

if [[ "$1" == "exec" && "$2" == "node" && "$3" == "scripts/tests/verify_playwright_launch.mjs" ]]; then
  echo "verify HOME=\${HOME} PWB=\${PLAYWRIGHT_BROWSERS_PATH:-<unset>}" >> "$trace_file"
  if [[ "\${PLAYWRIGHT_BROWSERS_PATH:-}" == "${REPO_CACHE_PATH}" ]]; then
    exit 42
  fi
  exit 0
fi

if [[ "$1" == "run" && "$2" == "test:dashboard:e2e:raw" ]]; then
  echo "run HOME=\${HOME} PWB=\${PLAYWRIGHT_BROWSERS_PATH:-<unset>}" >> "$trace_file"
  exit 0
fi

echo "unexpected pnpm args: $*" >> "$trace_file"
exit 98
`
  );

  const env = {
    ...process.env,
    PATH: `${fakeBinDir}:${process.env.PATH || ''}`,
    HOME: systemHome,
    PLAYWRIGHT_FORCE_LOCAL_HOME: '1',
    FAKE_COREPACK_TRACE_FILE: traceFile
  };
  delete env.PLAYWRIGHT_BROWSERS_PATH;
  delete env.PLAYWRIGHT_SANDBOX_ALLOW_SKIP;

  let stdout = '';
  try {
    stdout = execFileSync('bash', [RUNNER_PATH], {
      cwd: ROOT_DIR,
      env,
      encoding: 'utf8'
    });
  } finally {
    // Keep trace file available for assertions even when invocation fails.
  }

  const traceLines = fs
    .readFileSync(traceFile, 'utf8')
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean);
  const verifyLines = traceLines.filter((line) => line.startsWith('verify '));
  assert.equal(verifyLines.length, 3, 'expected three verify preflight attempts');
  assert.match(
    verifyLines[0],
    new RegExp(`^verify HOME=${escapeRegExp(REPO_LOCAL_HOME_PATH)} PWB=${escapeRegExp(REPO_CACHE_PATH)}$`)
  );
  assert.match(
    verifyLines[1],
    new RegExp(`^verify HOME=${escapeRegExp(systemHome)} PWB=${escapeRegExp(REPO_CACHE_PATH)}$`)
  );
  assert.match(
    verifyLines[2],
    new RegExp(`^verify HOME=${escapeRegExp(systemHome)} PWB=<unset>$`)
  );

  const runLine = traceLines.find((line) => line.startsWith('run '));
  assert.ok(runLine, 'expected e2e run invocation after successful preflight');
  assert.match(runLine, new RegExp(`^run HOME=${escapeRegExp(systemHome)} PWB=<unset>$`));

  assert.match(stdout, /retrying preflight with system HOME/i);
  assert.match(stdout, /retrying preflight with system browser cache/i);
  assert.match(stdout, /preflight succeeded with system HOME .* system Playwright browser cache/i);
});

test('playwright config only includes browser smoke specs in e2e runs', () => {
  const configPath = path.join(ROOT_DIR, 'playwright.config.mjs');
  const source = fs.readFileSync(configPath, 'utf8');
  assert.match(
    source,
    /testMatch:\s*["']\*\*\/\*\.spec\.js["']/,
    'playwright config should target *.spec.js so node unit tests are not re-run in browser suite'
  );
});

test('runner does not allow sandbox skip escape hatch in CI mode', () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'run-dashboard-e2e-ci-gate-'));
  const fakeBinDir = path.join(tempDir, 'bin');
  const systemHome = path.join(tempDir, 'system-home');
  const traceFile = path.join(tempDir, 'trace.log');
  fs.mkdirSync(fakeBinDir, { recursive: true });
  fs.mkdirSync(systemHome, { recursive: true });

  const fakeCorepackPath = path.join(fakeBinDir, 'corepack');
  writeExecutable(
    fakeCorepackPath,
    `#!/bin/bash
set -euo pipefail
trace_file="\${FAKE_COREPACK_TRACE_FILE:?}"

if [[ "$1" != "pnpm" ]]; then
  echo "unexpected corepack args: $*" >> "$trace_file"
  exit 99
fi
shift

if [[ "$1" == "exec" && "$2" == "node" && "$3" == "-e" ]]; then
  if [[ -n "\${PLAYWRIGHT_BROWSERS_PATH:-}" ]]; then
    echo "\${PLAYWRIGHT_BROWSERS_PATH}/chromium-not-installed"
  else
    echo "\${HOME}/Library/Caches/ms-playwright/chromium-not-installed"
  fi
  exit 0
fi

if [[ "$1" == "exec" && "$2" == "playwright" && "$3" == "install" && "$4" == "chromium" ]]; then
  echo "install HOME=\${HOME} PWB=\${PLAYWRIGHT_BROWSERS_PATH:-<unset>}" >> "$trace_file"
  exit 0
fi

if [[ "$1" == "exec" && "$2" == "node" && "$3" == "scripts/tests/verify_playwright_launch.mjs" ]]; then
  echo "verify HOME=\${HOME} PWB=\${PLAYWRIGHT_BROWSERS_PATH:-<unset>}" >> "$trace_file"
  exit 42
fi

if [[ "$1" == "run" && "$2" == "test:dashboard:e2e:raw" ]]; then
  echo "run HOME=\${HOME} PWB=\${PLAYWRIGHT_BROWSERS_PATH:-<unset>}" >> "$trace_file"
  exit 0
fi

echo "unexpected pnpm args: $*" >> "$trace_file"
exit 98
`
  );

  const env = {
    ...process.env,
    PATH: `${fakeBinDir}:${process.env.PATH || ''}`,
    HOME: systemHome,
    PLAYWRIGHT_FORCE_LOCAL_HOME: '1',
    PLAYWRIGHT_SANDBOX_ALLOW_SKIP: '1',
    CI: '1',
    FAKE_COREPACK_TRACE_FILE: traceFile
  };
  delete env.PLAYWRIGHT_BROWSERS_PATH;

  let failed = false;
  let output = '';
  try {
    execFileSync('bash', [RUNNER_PATH], {
      cwd: ROOT_DIR,
      env,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'pipe']
    });
  } catch (error) {
    failed = true;
    output = `${error.stdout || ''}\n${error.stderr || ''}`;
    assert.equal(error.status, 42, 'runner should preserve preflight failure status in CI');
  }

  assert.equal(failed, true, 'runner should fail in CI instead of skipping e2e');
  assert.match(output, /PLAYWRIGHT_SANDBOX_ALLOW_SKIP=1 is not allowed in CI/i);

  const traceLines = fs
    .readFileSync(traceFile, 'utf8')
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean);
  assert.equal(
    traceLines.some((line) => line.startsWith('run ')),
    false,
    'browser suite must not start after CI skip-mode rejection'
  );
});
