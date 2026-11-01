#!/usr/bin/env python3
"""Supplement actionlint/zizmor with repository-specific fail-closed policy checks."""
import pathlib
import re
import yaml

ROOT = pathlib.Path(__file__).resolve().parents[1]
PIN = re.compile(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_./-]+@[0-9a-f]{40}")


def validate(path):
    # BaseLoader deliberately retains the YAML 1.2 Actions `on` key as a string.
    data = yaml.load(path.read_text(), Loader=yaml.BaseLoader)
    assert "on" in data and isinstance(data.get("jobs"), dict), f"{path}: invalid workflow"
    assert data.get("permissions") == {"contents": "read"}, f"{path}: broad default permissions"
    assert "pull_request_target" not in data["on"], f"{path}: privileged fork execution"
    assert "concurrency" in data, f"{path}: no concurrency control"
    for name, job in data["jobs"].items():
        assert "secrets" not in job, f"{path}/{name}: secrets are not needed for build-only jobs"
        assert "permissions" not in job or job["permissions"] == {"contents": "read"}, f"{path}/{name}: escalated permissions"
        if "uses" in job:
            assert job["uses"].startswith("./.github/workflows/"), f"{path}/{name}: external reusable workflow"
            assert (ROOT / job["uses"]).is_file(), f"{path}/{name}: missing reusable workflow"
        else:
            assert "timeout-minutes" in job, f"{path}/{name}: unbounded duration"
        for step in job.get("steps", []):
            action = step.get("uses")
            if action:
                assert PIN.fullmatch(action), f"{path}: unpinned action {action}"
                if action.startswith("actions/checkout@"):
                    assert step.get("with", {}).get("persist-credentials") == "false", f"{path}: checkout credential persistence"
            script = step.get("run", "")
            assert "${{" not in script, f"{path}: context interpolation in shell; pass validated env instead"
            assert not step.get("continue-on-error"), f"{path}: hidden failing security/build command"
    return data


if __name__ == "__main__":
    paths = sorted((ROOT / ".github/workflows").glob("*.yml"))
    for path in paths:
        validate(path)
    print(f"PASS: {len(paths)} workflows; SHA pins, credential boundaries, permissions, timeout/concurrency, local reuse and no shell-context interpolation")
