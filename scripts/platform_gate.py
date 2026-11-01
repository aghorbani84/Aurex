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
        "dioxus_mobile_feature": "dioxus/mobile" in app.get("features", {}).get("mobile", []),
        "dioxus_configuration": (ROOT / "apps/aurex-app/Dioxus.toml").is_file(),
        "android_storage_provider": (ROOT / "apps/aurex-app/src/mobile.rs").is_file() and (ROOT / "apps/aurex-app/android/src/main/kotlin/io/github/aghorbani84/aurex/storage/StoragePlugin.kt").is_file(),
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
            "note": "Prerequisites only. The mobile SAF source and explicit native build workflow now exist. This report never certifies an APK or runtime. See the actual build/permission/ABI/signature and emulator evidence; production signing and write operations remain separate gates."}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", default="artifacts/android/readiness.json")
    args = parser.parse_args()
    report = inspect_android()
    path = pathlib.Path(args.output)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(report, indent=2) + "\n")
    print(path.read_text())
    # Success here means prerequisites only, never an APK/runtime pass.
    raise SystemExit(0 if report["state"] == "PREREQUISITES_ONLY" else 2)


if __name__ == "__main__":
    main()
