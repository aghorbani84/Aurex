#!/usr/bin/env python3
"""Fail closed on absent Android architecture/tooling instead of manufacturing an APK."""
import argparse
import json
import os
import pathlib
import shutil
import subprocess
import tomllib

ROOT = pathlib.Path(__file__).resolve().parents[1]


def inspect_android():
    app = tomllib.loads((ROOT / "apps/aurex-app/Cargo.toml").read_text())
    dioxus = app["dependencies"]["dioxus"]
    checks = {
        "dioxus_mobile_feature": "mobile" in dioxus.get("features", []) or "mobile" in app.get("features", {}),
        "dioxus_configuration": (ROOT / "apps/aurex-app/Dioxus.toml").is_file(),
        "android_storage_provider": (ROOT / "crates/aurex-platform-android/Cargo.toml").is_file(),
        "java": bool(shutil.which("java")),
        "dioxus_cli": bool(shutil.which("dx")),
        "sdk": bool(os.environ.get("ANDROID_HOME") and pathlib.Path(os.environ["ANDROID_HOME"]).is_dir()),
        "ndk": bool(os.environ.get("ANDROID_NDK_HOME") and pathlib.Path(os.environ["ANDROID_NDK_HOME"]).is_dir()),
    }
    rustc = shutil.which("rustc")
    for target in ["aarch64-linux-android", "x86_64-linux-android"]:
        path = subprocess.run([rustc, "--print", "target-libdir", "--target", target], capture_output=True, text=True) if rustc else None
        checks[target] = bool(path and path.returncode == 0 and pathlib.Path(path.stdout.strip()).is_dir())
    return {"platform": "Android", "state": "BLOCKED" if not all(checks.values()) else "PREREQUISITES_ONLY",
            "checks": checks, "packaging_verified": False,
            "note": "Readiness checks only. Provider runtime, package identifier/manifest/SDK versions/signing and APK packaging require a separate implemented mobile slice. No APK/AAB or broad storage permission is fabricated."}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", default="artifacts/android/readiness.json")
    args = parser.parse_args()
    report = inspect_android()
    path = pathlib.Path(args.output)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(report, indent=2) + "\n")
    print(path.read_text())
    # Even all tools present is not an app build. Keep the packaging gate closed
    # until an implemented mobile build and runtime acceptance replaces this gate.
    raise SystemExit(2)


if __name__ == "__main__":
    main()
