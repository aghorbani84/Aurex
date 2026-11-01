#!/usr/bin/env python3
"""Run every audit independently, retain failures and expose bounded diagnostics.

Never turn an advisory/tool failure into success, or skip later graphs after the
first failure. GitHub annotations make failure detail accessible even when this
sandbox cannot download Actions logs/artifacts from external storage hosts.
"""
import json
import os
import pathlib
import re
import subprocess
import uuid

ROOT = pathlib.Path(__file__).resolve().parents[1]


def commands():
    scans = []
    for scope, manifest, lock in [("core", "Cargo.toml", "Cargo.lock"),
                                  ("desktop", "apps/aurex-app/Cargo.toml", "apps/aurex-app/Cargo.lock")]:
        scans += [(scope + "-audit", ["cargo", "audit", "--file", lock]),
                  (scope + "-deny", ["cargo", "deny", "--manifest-path", manifest, "--config", "deny.toml", "--locked",
                                    "check", "advisories", "licenses", "bans", "sources"])]
    return scans


def collect(runner=None):
    runner = runner or subprocess.run
    results = []
    for name, command in commands():
        try:
            result = runner(command, cwd=ROOT, text=True, capture_output=True, timeout=900)
            code, output = result.returncode, result.stdout + result.stderr
        except (OSError, subprocess.TimeoutExpired) as error:
            code, output = 1, str(error)
        # Keep all finding headers/IDs even if the final verbose inclusion tree is huge.
        summary = [line for line in output.splitlines() if
                   re.search(r"^error|RUSTSEC-|(?:ID|Crate|Version|Title|Solution):", line)]
        # Disable workflow command interpretation while printing tool-controlled text.
        stop = "audit-" + uuid.uuid4().hex
        if os.environ.get("GITHUB_ACTIONS") == "true":
            print(f"::stop-commands::{stop}", flush=True)
        print(f"\n=== {name}: exit {code} ===\n{output}", flush=True)
        if os.environ.get("GITHUB_ACTIONS") == "true":
            print(f"::{stop}::", flush=True)
            if code:
                excerpt = (name + ": " + "\n".join(summary)[:16000] + "\nFIRST DIAGNOSTIC CONTEXT:\n" + output[:6000] + "\nLAST DIAGNOSTIC CONTEXT:\n" + output[-3000:]).replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A")
                print(f"::error title=Dependency audit failure::{excerpt}", flush=True)
        results.append({"check": name, "command": command, "exit_code": code, "finding_summary": summary[:200], "diagnostic_tail": output[-16000:]})
    return results


def main():
    results = collect()
    path = ROOT / "artifacts/security/dependencies.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps({"checks": results, "passed": all(r["exit_code"] == 0 for r in results)}, indent=2) + "\n")
    raise SystemExit(0 if all(r["exit_code"] == 0 for r in results) else 1)


if __name__ == "__main__":
    main()
