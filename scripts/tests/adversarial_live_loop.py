#!/usr/bin/env python3
"""Resilient live-loop orchestration for adversarial simulation profiles."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Any, Dict, List, Tuple


TRANSIENT_MARKERS = (
    "spin server was not ready",
    "timed out",
    "timeout",
    "connection reset",
    "connection refused",
    "temporary failure",
    "http error 429",
    "http error 500",
    "http error 502",
    "http error 503",
    "http error 504",
)

MEANINGFUL_DEFENSE_REASON_PREFIXES = (
    "honeypot",
    "rate",
    "geo_policy_",
    "not_a_bot_",
    "challenge_",
    "cdp_",
    "edge_fingerprint_",
    "external_fingerprint_",
    "tarpit_",
    "banned",
    "maze_",
)

ADMIN_NOISE_REASON_PREFIXES = (
    "admin_",
    "config_",
    "provider_selection_update",
    "shadow_mode_toggle",
)


def parse_non_negative_int(raw_value: str, name: str) -> int:
    try:
        value = int(str(raw_value).strip())
    except Exception as exc:
        raise ValueError(f"{name} must be an integer >= 0") from exc
    if value < 0:
        raise ValueError(f"{name} must be an integer >= 0")
    return value


def classify_failure(output_text: str) -> str:
    lowered = str(output_text or "").lower()
    for marker in TRANSIENT_MARKERS:
        if marker in lowered:
            return "transient"
    return "fatal"


def has_meaningful_defense_events(reasons: List[str]) -> Tuple[bool, List[str]]:
    normalized = [str(reason or "").strip().lower() for reason in reasons]
    filtered = [reason for reason in normalized if reason]
    matched = []
    for reason in filtered:
        if any(reason.startswith(prefix) for prefix in MEANINGFUL_DEFENSE_REASON_PREFIXES):
            matched.append(reason)
    if matched:
        return True, sorted(set(matched))
    # If everything is admin/config noise, treat as low-quality cycle.
    if filtered and all(
        any(reason.startswith(prefix) for prefix in ADMIN_NOISE_REASON_PREFIXES)
        for reason in filtered
    ):
        return False, []
    return False, []


def load_report(path: Path) -> Dict[str, Any]:
    if not path.exists():
        raise RuntimeError(f"report not found: {path}")
    try:
        parsed = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise RuntimeError(f"invalid report JSON at {path}") from exc
    if not isinstance(parsed, dict):
        raise RuntimeError(f"report must be a JSON object: {path}")
    return parsed


def read_recent_event_reasons(report: Dict[str, Any]) -> List[str]:
    before = report.get("monitoring_after")
    if not isinstance(before, dict):
        return []
    reasons = before.get("recent_event_reasons")
    if not isinstance(reasons, list):
        return []
    return [str(item or "") for item in reasons]


def read_tarpit_metrics(report: Dict[str, Any]) -> Dict[str, int]:
    snapshot = report.get("monitoring_after")
    if not isinstance(snapshot, dict):
        return {}
    tarpit = snapshot.get("tarpit")
    if not isinstance(tarpit, dict):
        tarpit = {}
    metrics_source = tarpit
    out = {
        "activations_progressive": int(
            metrics_source.get("metrics", {})
            .get("activations", {})
            .get("progressive", 0)
        ),
        "progress_advanced": int(
            metrics_source.get("metrics", {})
            .get("progress_outcomes", {})
            .get("advanced", 0)
        ),
        "fallback_maze": int(
            metrics_source.get("metrics", {})
            .get("budget_outcomes", {})
            .get("fallback_maze", 0)
        ),
        "fallback_block": int(
            metrics_source.get("metrics", {})
            .get("budget_outcomes", {})
            .get("fallback_block", 0)
        ),
        "escalation_short_ban": int(
            metrics_source.get("metrics", {})
            .get("escalation_outcomes", {})
            .get("short_ban", 0)
        ),
        "escalation_block": int(
            metrics_source.get("metrics", {})
            .get("escalation_outcomes", {})
            .get("block", 0)
        ),
    }
    return out


def run_simulation_cycle(args: argparse.Namespace) -> subprocess.CompletedProcess[str]:
    command = [
        "python3",
        "scripts/tests/adversarial_simulation_runner.py",
        "--manifest",
        args.manifest,
        "--profile",
        args.profile,
        "--report",
        args.report,
    ]
    env = dict(os.environ)
    env["SHUMA_ADVERSARIAL_PRESERVE_STATE"] = "1" if args.preserve_state else "0"
    env["SHUMA_ADVERSARIAL_ROTATE_IPS"] = "1" if args.rotate_ips else "0"
    return subprocess.run(command, capture_output=True, text=True, env=env, check=False)


def validate_profile(args: argparse.Namespace) -> None:
    command = [
        "python3",
        "scripts/tests/adversarial_simulation_runner.py",
        "--manifest",
        args.manifest,
        "--profile",
        args.profile,
        "--validate-only",
    ]
    result = subprocess.run(command, capture_output=True, text=True, check=False)
    if result.returncode != 0:
        raise RuntimeError(
            f"invalid profile '{args.profile}'; supported profiles are defined in {args.manifest}"
        )


def backoff_seconds(base_seconds: int, max_seconds: int, retry_number: int) -> int:
    if retry_number <= 0:
        return 0
    return min(max_seconds, base_seconds * (2 ** (retry_number - 1)))


def main() -> int:
    parser = argparse.ArgumentParser(description="Run resilient adversarial live simulation loops")
    parser.add_argument("--manifest", required=True, help="Manifest path")
    parser.add_argument("--profile", required=True, help="Profile name")
    parser.add_argument("--runs", default="0", help="Successful cycle count target (0 means infinite)")
    parser.add_argument("--pause-seconds", default="2", help="Pause between successful cycles")
    parser.add_argument("--report", default="scripts/tests/adversarial/latest_report.json", help="Report path")
    parser.add_argument("--cleanup-mode", default="0", help="0=preserve state; 1=cleanup each cycle")
    parser.add_argument("--fatal-cycle-limit", default="3", help="Consecutive fatal cycles before stop")
    parser.add_argument(
        "--transient-retry-limit",
        default="4",
        help="Consecutive transient retries before counting as one fatal cycle",
    )
    parser.add_argument("--backoff-base-seconds", default="2", help="Transient retry backoff base")
    parser.add_argument("--backoff-max-seconds", default="30", help="Transient retry backoff max")
    parser.add_argument("--preserve-state", default="1", help="Preserve state flag when cleanup-mode is 0")
    parser.add_argument("--rotate-ips", default="1", help="Rotate scenario IPs between runs")
    args = parser.parse_args()

    try:
        target_runs = parse_non_negative_int(args.runs, "runs")
        pause_seconds = parse_non_negative_int(args.pause_seconds, "pause_seconds")
        fatal_cycle_limit = max(1, parse_non_negative_int(args.fatal_cycle_limit, "fatal_cycle_limit"))
        transient_retry_limit = max(
            1, parse_non_negative_int(args.transient_retry_limit, "transient_retry_limit")
        )
        backoff_base_seconds = max(
            1, parse_non_negative_int(args.backoff_base_seconds, "backoff_base_seconds")
        )
        backoff_max_seconds = max(
            backoff_base_seconds,
            parse_non_negative_int(args.backoff_max_seconds, "backoff_max_seconds"),
        )
        cleanup_mode = parse_non_negative_int(args.cleanup_mode, "cleanup_mode")
        if cleanup_mode not in {0, 1}:
            raise ValueError("cleanup_mode must be 0 or 1")
        preserve_state = parse_non_negative_int(args.preserve_state, "preserve_state")
        rotate_ips = parse_non_negative_int(args.rotate_ips, "rotate_ips")
        args.preserve_state = False if cleanup_mode == 1 else bool(preserve_state)
        args.rotate_ips = bool(rotate_ips)
        validate_profile(args)
    except Exception as exc:
        print(f"[adversarial-live] invalid configuration: {exc}", file=sys.stderr)
        return 2

    print(
        "[adversarial-live] config "
        f"profile={args.profile} runs={target_runs} pause_seconds={pause_seconds} "
        f"preserve_state={1 if args.preserve_state else 0} rotate_ips={1 if args.rotate_ips else 0} "
        f"fatal_cycle_limit={fatal_cycle_limit} transient_retry_limit={transient_retry_limit}"
    )

    successful_cycles = 0
    attempted_cycles = 0
    consecutive_fatal_cycles = 0
    consecutive_transient_retries = 0
    last_terminal_reason = ""

    while True:
        attempted_cycles += 1
        print(f"[adversarial-live] cycle={attempted_cycles} status=running")
        result = run_simulation_cycle(args)
        output = "\n".join([result.stdout or "", result.stderr or ""]).strip()

        if result.returncode == 0:
            consecutive_transient_retries = 0
            consecutive_fatal_cycles = 0
            successful_cycles += 1
            try:
                report = load_report(Path(args.report))
                reasons = read_recent_event_reasons(report)
                meaningful, matched_reasons = has_meaningful_defense_events(reasons)
                tarpit_metrics = read_tarpit_metrics(report)
                print(
                    "[adversarial-live] cycle={} status=pass meaningful_events={} matched={} tarpit={}".format(
                        attempted_cycles,
                        "yes" if meaningful else "no",
                        ",".join(matched_reasons[:8]) if matched_reasons else "-",
                        json.dumps(tarpit_metrics, sort_keys=True),
                    )
                )
                if not meaningful:
                    last_terminal_reason = "meaningful_defense_events_missing"
                    consecutive_fatal_cycles += 1
                    print(
                        "[adversarial-live] cycle={} classification=fatal reason={} "
                        "consecutive_fatal={}/{}".format(
                            attempted_cycles,
                            last_terminal_reason,
                            consecutive_fatal_cycles,
                            fatal_cycle_limit,
                        )
                    )
                    if consecutive_fatal_cycles >= fatal_cycle_limit:
                        print(
                            "[adversarial-live] terminal_failure reason={} consecutive_fatal={}".format(
                                last_terminal_reason, consecutive_fatal_cycles
                            ),
                            file=sys.stderr,
                        )
                        return 1
                else:
                    if target_runs > 0 and successful_cycles >= target_runs:
                        print(
                            f"[adversarial-live] completed successful_cycles={successful_cycles} attempted_cycles={attempted_cycles}"
                        )
                        return 0
                    if pause_seconds > 0:
                        time.sleep(pause_seconds)
                continue
            except Exception as exc:
                output = f"{output}\nreport_parse_error={exc}".strip()
                result = subprocess.CompletedProcess(result.args, 1, result.stdout, output)

        classification = classify_failure(output)
        if classification == "transient":
            consecutive_transient_retries += 1
            last_terminal_reason = "transient_runner_failure"
            if consecutive_transient_retries <= transient_retry_limit:
                retry_backoff = backoff_seconds(
                    backoff_base_seconds, backoff_max_seconds, consecutive_transient_retries
                )
                print(
                    "[adversarial-live] cycle={} classification=transient retry_count={} backoff_seconds={} reason={}".format(
                        attempted_cycles,
                        consecutive_transient_retries,
                        retry_backoff,
                        last_terminal_reason,
                    )
                )
                if retry_backoff > 0:
                    time.sleep(retry_backoff)
                attempted_cycles -= 1
                continue
            print(
                "[adversarial-live] cycle={} classification=transient retry_limit_exhausted={} reason={}".format(
                    attempted_cycles, transient_retry_limit, last_terminal_reason
                )
            )
            consecutive_transient_retries = 0
            consecutive_fatal_cycles += 1
        else:
            consecutive_transient_retries = 0
            consecutive_fatal_cycles += 1
            last_terminal_reason = "fatal_runner_failure"
            print(
                "[adversarial-live] cycle={} classification=fatal retry_count=0 reason={}".format(
                    attempted_cycles, last_terminal_reason
                )
            )

        if consecutive_fatal_cycles >= fatal_cycle_limit:
            print(
                "[adversarial-live] terminal_failure reason={} consecutive_fatal={} attempted_cycles={}".format(
                    last_terminal_reason,
                    consecutive_fatal_cycles,
                    attempted_cycles,
                ),
                file=sys.stderr,
            )
            return 1

        if pause_seconds > 0:
            time.sleep(pause_seconds)


if __name__ == "__main__":
    sys.exit(main())
