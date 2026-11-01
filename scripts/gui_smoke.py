#!/usr/bin/env python3
"""Native graphical fixture test. Run under xvfb-run; no pre-existing user files."""
import hashlib
import json
import os
import pathlib
import subprocess
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]


def main():
    report = {"platform": "Linux x86_64", "passed": False, "display": bool(os.environ.get("DISPLAY")),
              "scope": "Native WebView DOM, navigation/tabs/filter/preview/read-only controls and graceful close; not full runtime certification"}
    path = ROOT / "artifacts/desktop/runtime.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    try:
        if not report["display"]:
            raise ValueError("a native display (or xvfb-run) is required; no pretend headless pass")
        subprocess.run(["cargo", "build", "--manifest-path", "apps/aurex-app/Cargo.toml", "--release", "--locked", "--features", "gui-smoke"], cwd=ROOT, check=True, timeout=1800)
        binary = ROOT / "apps/aurex-app/target/release/aurex-app"
        with tempfile.TemporaryDirectory(prefix="aurex-gui-fixture-") as temp:
            root = pathlib.Path(temp)
            payload = b"AUREX native runtime verification\n"
            (root / "aurex-smoke.txt").write_bytes(payload)
            (root / "nested").mkdir(mode=0o700)
            (root / "nested/inside.txt").write_bytes(b"inside\n")
            before = {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest() for p in root.rglob("*") if p.is_file()}
            result = subprocess.run([str(binary), temp, "--smoke-test"], cwd=ROOT, text=True, capture_output=True, timeout=60)
            # Fixture process output is not accepted as workflow commands.
            report["exit_code"] = result.returncode
            report["marker"] = "AUREX_GUI_SMOKE_PASS:" in result.stdout
            report["diagnostic_tail"] = (result.stdout + result.stderr)[-8000:]
            after = {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest() for p in root.rglob("*") if p.is_file()}
            if result.returncode or not report["marker"] or before != after:
                raise ValueError("native GUI assertions/exit or fixture integrity failed; inspect runtime.json")
            report["passed"] = True
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        report["error"] = str(error)
        if isinstance(error, subprocess.TimeoutExpired):
            def text(value):
                return value.decode("utf-8", errors="replace") if isinstance(value, bytes) else value or ""
            report["diagnostic_tail"] = (text(error.stdout) + text(error.stderr))[-8000:]
    path.write_text(json.dumps(report, indent=2) + "\n")
    print("PASS: native GUI fixture" if report["passed"] else "FAIL: native GUI fixture; see runtime.json")
    if not report["passed"]:
        if os.environ.get("GITHUB_ACTIONS") == "true":
            detail = (report.get("error", "Native fixture failed") + "\n" + report.get("diagnostic_tail", ""))[-10000:]
            print("::error title=Linux native runtime test failed::" + detail.replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A"))
        raise SystemExit(1)


if __name__ == "__main__":
    main()
