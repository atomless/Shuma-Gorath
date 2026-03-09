#!/usr/bin/env python3

"""Frontier-lane unavailability threshold policy enforcement."""

from __future__ import annotations

import argparse
import json
import os
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any, Dict, List, Optional


TRACKER_TITLE = "Frontier Lane Degradation Tracker"
TRACKER_LABEL = "frontier-lane-tracker"
REFRESH_TITLE = "Frontier Supported Model Refresh Required"
REFRESH_LABEL = "frontier-model-refresh"
TRACKER_MARKER = "<!-- frontier-lane-tracker -->"


class GitHubRequestError(RuntimeError):
    def __init__(self, method: str, path: str, status_code: int, detail: str):
        self.method = method
        self.path = path
        self.status_code = int(status_code)
        self.detail = detail
        super().__init__(
            f"github API {method} {path} failed: status={status_code} body={detail[:300]}"
        )


def load_json(path: Path) -> Dict[str, Any]:
    if not path.exists():
        raise ValueError(f"missing JSON file: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise ValueError(f"invalid JSON file: {path}") from exc
    if not isinstance(payload, dict):
        raise ValueError(f"JSON payload must be object: {path}")
    return payload


def save_json(path: Path, payload: Dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2), encoding="utf-8")


def parse_int(value: str, default: int) -> int:
    try:
        return int(str(value).strip())
    except Exception:
        return default


def parse_tracker_state_from_body(body: str) -> Dict[str, Any]:
    state = {
        "consecutive_degraded_runs": 0,
        "first_degraded_at_unix": 0,
        "last_observed_status": "",
        "last_checked_unix": 0,
    }
    if not body:
        return state
    for raw_line in body.splitlines():
        line = raw_line.strip()
        if ":" not in line:
            continue
        key, value = line.split(":", 1)
        key = key.strip()
        value = value.strip()
        if key == "consecutive_degraded_runs":
            state[key] = parse_int(value, 0)
        elif key == "first_degraded_at_unix":
            state[key] = parse_int(value, 0)
        elif key == "last_checked_unix":
            state[key] = parse_int(value, 0)
        elif key == "last_observed_status":
            state[key] = value
    return state


def render_tracker_body(state: Dict[str, Any]) -> str:
    return "\n".join(
        [
            TRACKER_MARKER,
            "",
            "Tracks protected-lane frontier availability for model-refresh threshold policy.",
            "",
            f"consecutive_degraded_runs: {int(state.get('consecutive_degraded_runs') or 0)}",
            f"first_degraded_at_unix: {int(state.get('first_degraded_at_unix') or 0)}",
            f"last_observed_status: {str(state.get('last_observed_status') or '')}",
            f"last_checked_unix: {int(state.get('last_checked_unix') or 0)}",
            "",
            "Threshold policy: open/assign supported-model refresh action when frontier lane is degraded for 10 consecutive protected-lane runs or 7 days (whichever occurs first).",
        ]
    )


def update_tracker_state(
    previous: Dict[str, Any], lane_status: str, now_unix: int
) -> Dict[str, Any]:
    degraded = lane_status != "ok"
    prior_runs = int(previous.get("consecutive_degraded_runs") or 0)
    prior_first = int(previous.get("first_degraded_at_unix") or 0)

    if degraded:
        consecutive = prior_runs + 1
        first_degraded = prior_first if prior_first > 0 else now_unix
    else:
        consecutive = 0
        first_degraded = 0

    return {
        "consecutive_degraded_runs": consecutive,
        "first_degraded_at_unix": first_degraded,
        "last_observed_status": lane_status,
        "last_checked_unix": now_unix,
    }


def threshold_reached(
    state: Dict[str, Any], max_consecutive_runs: int, max_degraded_days: int, now_unix: int
) -> bool:
    consecutive = int(state.get("consecutive_degraded_runs") or 0)
    first_degraded = int(state.get("first_degraded_at_unix") or 0)
    if consecutive >= max_consecutive_runs:
        return True
    if first_degraded <= 0:
        return False
    return (now_unix - first_degraded) >= max_degraded_days * 86_400


def github_request(
    method: str,
    repo: str,
    token: str,
    path: str,
    payload: Optional[Dict[str, Any]] = None,
) -> Dict[str, Any]:
    url = f"https://api.github.com/repos/{repo}{path}"
    headers = {
        "Authorization": f"Bearer {token}",
        "Accept": "application/vnd.github+json",
        "X-GitHub-Api-Version": "2022-11-28",
    }
    data = None
    if payload is not None:
        data = json.dumps(payload).encode("utf-8")
        headers["Content-Type"] = "application/json"
    request = urllib.request.Request(url=url, method=method, data=data, headers=headers)
    try:
        with urllib.request.urlopen(request, timeout=20) as response:
            body = response.read().decode("utf-8", errors="replace")
            if not body.strip():
                return {}
            parsed = json.loads(body)
            if isinstance(parsed, dict):
                return parsed
            return {"items": parsed} if isinstance(parsed, list) else {}
    except urllib.error.HTTPError as exc:
        detail = exc.read().decode("utf-8", errors="replace")
        raise GitHubRequestError(method, path, exc.code, detail)


def github_issues_disabled(exc: Exception) -> bool:
    return isinstance(exc, GitHubRequestError) and exc.status_code == 410 and (
        "Issues has been disabled" in exc.detail
    )


def github_list_open_issues(repo: str, token: str, label: str) -> List[Dict[str, Any]]:
    query = urllib.parse.urlencode({"state": "open", "labels": label, "per_page": 100})
    payload = github_request("GET", repo, token, f"/issues?{query}")
    if isinstance(payload.get("items"), list):
        return [item for item in payload["items"] if isinstance(item, dict)]
    if isinstance(payload, dict) and "number" in payload:
        return [payload]
    return []


def github_find_issue_by_title(
    issues: List[Dict[str, Any]], title: str
) -> Optional[Dict[str, Any]]:
    for issue in issues:
        if str(issue.get("title") or "").strip() == title:
            return issue
    return None


def github_create_issue(
    repo: str, token: str, title: str, body: str, labels: List[str]
) -> Dict[str, Any]:
    return github_request(
        "POST",
        repo,
        token,
        "/issues",
        payload={
            "title": title,
            "body": body,
            "labels": labels,
        },
    )


def github_update_issue(repo: str, token: str, issue_number: int, payload: Dict[str, Any]) -> Dict[str, Any]:
    return github_request(
        "PATCH",
        repo,
        token,
        f"/issues/{issue_number}",
        payload=payload,
    )


def github_assign_issue(repo: str, token: str, issue_number: int, assignee: str) -> None:
    if not assignee.strip():
        return
    try:
        github_request(
            "POST",
            repo,
            token,
            f"/issues/{issue_number}/assignees",
            payload={"assignees": [assignee]},
        )
    except RuntimeError as exc:
        # Keep policy run actionable even when owner assignment cannot be applied.
        print(f"[frontier-policy] warning: failed to assign {assignee}: {exc}")


def refresh_issue_body(state: Dict[str, Any], lane_status: Dict[str, Any]) -> str:
    advisory = str(lane_status.get("advisory") or "")
    return "\n".join(
        [
            "Frontier lane remained degraded past threshold policy.",
            "",
            f"- consecutive_degraded_runs: {int(state.get('consecutive_degraded_runs') or 0)}",
            f"- first_degraded_at_unix: {int(state.get('first_degraded_at_unix') or 0)}",
            f"- last_observed_status: {str(state.get('last_observed_status') or '')}",
            f"- advisory: {advisory}",
            "",
            "Required actions:",
            "1. Refresh supported frontier model list/documentation.",
            "2. Validate provider credentials/quotas and restore healthy protected-lane probes.",
            "3. Record remediation owner and ETA in this issue.",
        ]
    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Evaluate protected-lane frontier degraded-threshold policy and optional GitHub action creation."
    )
    parser.add_argument(
        "--status",
        default="scripts/tests/adversarial/frontier_lane_status.json",
        help="Path to frontier lane status JSON",
    )
    parser.add_argument(
        "--output",
        default="scripts/tests/adversarial/frontier_unavailability_policy.json",
        help="Output JSON summary path",
    )
    parser.add_argument(
        "--max-consecutive-runs",
        type=int,
        default=10,
        help="Consecutive degraded protected-lane runs threshold",
    )
    parser.add_argument(
        "--max-degraded-days",
        type=int,
        default=7,
        help="Elapsed degraded days threshold",
    )
    parser.add_argument(
        "--owner",
        default=os.environ.get("FRONTIER_MODEL_REFRESH_OWNER", ""),
        help="GitHub username to assign refresh action to when threshold is reached",
    )
    parser.add_argument(
        "--enable-github",
        action="store_true",
        help="Enable GitHub issue tracking/creation using GITHUB_REPOSITORY + GITHUB_TOKEN",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    lane_status = load_json(Path(args.status))
    now_unix = int(time.time())
    lane_status_code = str(lane_status.get("status") or "unknown")
    tracker_state = update_tracker_state(
        previous={},
        lane_status=lane_status_code,
        now_unix=now_unix,
    )
    tracker_issue_url = ""
    refresh_issue_url = ""
    github_issue_tracking_status = "not_requested"
    github_issue_tracking_note = ""

    if args.enable_github:
        github_issue_tracking_status = "enabled"
        repo = str(os.environ.get("GITHUB_REPOSITORY", "")).strip()
        token = str(os.environ.get("GITHUB_TOKEN", "")).strip()
        if not repo or not token:
            raise RuntimeError(
                "--enable-github requires GITHUB_REPOSITORY and GITHUB_TOKEN"
            )
        try:
            tracker_issues = github_list_open_issues(repo, token, TRACKER_LABEL)
            tracker_issue = github_find_issue_by_title(tracker_issues, TRACKER_TITLE)
            previous_state = parse_tracker_state_from_body(
                str(dict(tracker_issue or {}).get("body") or "")
            )
            tracker_state = update_tracker_state(
                previous=previous_state,
                lane_status=lane_status_code,
                now_unix=now_unix,
            )
            tracker_body = render_tracker_body(tracker_state)

            if tracker_issue:
                issue_number = int(tracker_issue.get("number") or 0)
                updated = github_update_issue(
                    repo,
                    token,
                    issue_number,
                    {"body": tracker_body},
                )
                tracker_issue_url = str(updated.get("html_url") or "")
            elif lane_status_code != "ok":
                created = github_create_issue(
                    repo,
                    token,
                    title=TRACKER_TITLE,
                    body=tracker_body,
                    labels=[TRACKER_LABEL],
                )
                tracker_issue_url = str(created.get("html_url") or "")

            reached = threshold_reached(
                tracker_state,
                max_consecutive_runs=max(1, int(args.max_consecutive_runs)),
                max_degraded_days=max(1, int(args.max_degraded_days)),
                now_unix=now_unix,
            )
            if reached and lane_status_code != "ok":
                refresh_issues = github_list_open_issues(repo, token, REFRESH_LABEL)
                refresh_issue = github_find_issue_by_title(refresh_issues, REFRESH_TITLE)
                refresh_body = refresh_issue_body(tracker_state, lane_status)
                if refresh_issue:
                    issue_number = int(refresh_issue.get("number") or 0)
                    updated = github_update_issue(
                        repo,
                        token,
                        issue_number,
                        {"body": refresh_body},
                    )
                    refresh_issue_url = str(updated.get("html_url") or "")
                    github_assign_issue(repo, token, issue_number, args.owner)
                else:
                    created = github_create_issue(
                        repo,
                        token,
                        title=REFRESH_TITLE,
                        body=refresh_body,
                        labels=[REFRESH_LABEL],
                    )
                    issue_number = int(created.get("number") or 0)
                    refresh_issue_url = str(created.get("html_url") or "")
                    github_assign_issue(repo, token, issue_number, args.owner)
        except GitHubRequestError as exc:
            if not github_issues_disabled(exc):
                raise
            github_issue_tracking_status = "issues_disabled"
            github_issue_tracking_note = (
                "GitHub Issues are disabled for this repository; frontier policy remains artifact-only."
            )
            print(f"[frontier-policy] warning: {github_issue_tracking_note}")

    reached = threshold_reached(
        tracker_state,
        max_consecutive_runs=max(1, int(args.max_consecutive_runs)),
        max_degraded_days=max(1, int(args.max_degraded_days)),
        now_unix=now_unix,
    )
    action_required = reached and lane_status_code != "ok"
    summary = {
        "schema_version": "frontier-unavailability-policy.v1",
        "generated_at_unix": now_unix,
        "lane_status": lane_status_code,
        "tracker_state": tracker_state,
        "threshold": {
            "max_consecutive_runs": int(args.max_consecutive_runs),
            "max_degraded_days": int(args.max_degraded_days),
            "reached": reached,
        },
        "action_required": action_required,
        "tracker_issue_url": tracker_issue_url,
        "refresh_issue_url": refresh_issue_url,
        "github_issue_tracking_status": github_issue_tracking_status,
        "github_issue_tracking_note": github_issue_tracking_note,
    }
    save_json(Path(args.output), summary)

    print(
        "[frontier-policy] status={} consecutive={} first_degraded_at={} threshold_reached={} action_required={}".format(
            lane_status_code,
            tracker_state.get("consecutive_degraded_runs"),
            tracker_state.get("first_degraded_at_unix"),
            reached,
            action_required,
        )
    )
    if tracker_issue_url:
        print(f"[frontier-policy] tracker_issue={tracker_issue_url}")
    if refresh_issue_url:
        print(f"[frontier-policy] refresh_issue={refresh_issue_url}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
