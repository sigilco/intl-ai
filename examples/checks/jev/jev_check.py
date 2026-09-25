#!/usr/bin/env python3
"""intl-ai exec check (protocol v1) bridging to TypeSafe's Jev.

Reads one JSONL request on stdin, scores each item against the Jev HTTP
API, prints one JSONL response on stdout. stdlib only.

Config:

    [[checks]]
    exec = "python3"
    args = ["examples/checks/jev/jev_check.py"]

Env:

    TYPESAFE_API_KEY   required, Jev API credential
    JEV_API_URL        default https://jev.systemone.example/v1/score
    JEV_THRESHOLD      flag findings below this score, default 0.8
"""

import json
import os
import sys
import urllib.request

API_URL = os.environ.get("JEV_API_URL", "https://jev.systemone.example/v1/score")
THRESHOLD = float(os.environ.get("JEV_THRESHOLD", "0.8"))
TIMEOUT = 30


def fail(code: str, detail: str) -> None:
    print(json.dumps({"v": 1, "error": f"{code}:{detail}"}))
    sys.exit(0)


def score_item(api_key: str, item: dict, locale: str) -> float:
    body = json.dumps(
        {
            "source": item.get("source"),
            "translation": item.get("target"),
            "locale": locale,
        }
    ).encode()
    req = urllib.request.Request(
        API_URL,
        data=body,
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {api_key}",
        },
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=TIMEOUT) as resp:
        data = json.loads(resp.read().decode())
    return float(data["score"])


def main() -> None:
    api_key = os.environ.get("TYPESAFE_API_KEY")
    if not api_key:
        return fail("config", "TYPESAFE_API_KEY is not set")

    raw = sys.stdin.readline()
    try:
        req = json.loads(raw)
    except json.JSONDecodeError as e:
        return fail("parse", f"bad request line: {e}")
    if req.get("v") != 1:
        return fail("parse", f"unsupported protocol version: {req.get('v')}")

    locale = req.get("target_locale", "")
    findings = []
    for item in req.get("items", []):
        if item.get("source") is None:
            continue
        try:
            score = score_item(api_key, item, locale)
        except Exception as e:  # noqa: BLE001 — surface as a check error
            return fail("transport", f"jev api: {e}")
        if score < THRESHOLD:
            findings.append(
                {
                    "key": item["key"],
                    "check": "jev",
                    "message": f"jev score {score:.2f} below {THRESHOLD}",
                }
            )

    print(json.dumps({"v": 1, "findings": findings}))


if __name__ == "__main__":
    main()
