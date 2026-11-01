#!/usr/bin/env python3
"""Build/verify a real engineering APK. Never manufacture a production signing key."""
import argparse
import collections
import hashlib
import json
import os
import pathlib
import re
import shutil
import struct
import subprocess
import uuid
import xml.etree.ElementTree as ET
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
APP = ROOT / "apps/aurex-app"
TARGETS = {"aarch64-linux-android": ("arm64-v8a", 183), "x86_64-linux-android": ("x86_64", 62)}


def elf_alignment(data, machine):
    if len(data) < 64 or data[:6] != b"\x7fELF\x02\x01" or struct.unpack_from("<H", data, 18)[0] != machine:
        raise ValueError("native library is not the requested 64-bit little-endian ELF architecture")
    offset = struct.unpack_from("<Q", data, 32)[0]
    size, count = struct.unpack_from("<HH", data, 54)
    if size < 56 or count == 0 or count > 4096 or offset + size * count > len(data):
        raise ValueError("invalid ELF program header table")
    loads = [struct.unpack_from("<IIQQQQQQ", data, offset + i * size) for i in range(count)
             if struct.unpack_from("<I", data, offset + i * size)[0] == 1]
    if not loads or any(header[7] < 16384 or header[7] & (header[7] - 1)
                        or header[5] > header[6] or header[2] + header[5] > len(data)
                        or header[2] % 16384 != header[3] % 16384 for header in loads):
        raise ValueError("native library is not linked for 16 KiB pages")
    return len(loads)


def verify_apk(apk, target, sdk):
    abi, machine = TARGETS[target]
    tool = sdk / "cmdline-tools/latest/bin/apkanalyzer"
    text = subprocess.check_output([str(tool), "manifest", "print", str(apk)], text=True)
    manifest = ET.fromstring(text)
    ns = "{http://schemas.android.com/apk/res/android}"
    if manifest.get("package") != "io.github.aghorbani84.aurex":
        raise ValueError("unexpected APK application identifier")
    application = manifest.find("application")
    if application is None or application.get(ns + "debuggable") != "true":
        raise ValueError("expected an explicitly debug-labelled engineering APK, not an invented signed release")
    if application.get(ns + "allowBackup") != "false":
        raise ValueError("persisted folder grants must not be included in Android backup")
    if application.get(ns + "usesCleartextTraffic") != "false":
        raise ValueError("cleartext traffic must remain disabled")
    prohibited = {"android.permission.MANAGE_EXTERNAL_STORAGE", "android.permission.READ_EXTERNAL_STORAGE",
                  "android.permission.WRITE_EXTERNAL_STORAGE", "android.permission.READ_MEDIA_IMAGES",
                  "android.permission.READ_MEDIA_VIDEO", "android.permission.READ_MEDIA_AUDIO"}
    permissions = {item.get(ns + "name") for item in manifest.findall("uses-permission")}
    if permissions & prohibited:
        raise ValueError("SAF app unexpectedly requests broad storage/media permissions")
    verified = []
    with zipfile.ZipFile(apk) as archive:
        if len(archive.namelist()) != len(set(archive.namelist())):
            raise ValueError("duplicate APK members")
        for member in archive.infolist():
            if member.filename.startswith("lib/") and member.filename.endswith(".so"):
                if member.file_size > 256 * 1024 * 1024 or not member.filename.startswith(f"lib/{abi}/"):
                    raise ValueError("unexpected native ABI or oversized library")
                elf_alignment(archive.read(member), machine)
                verified.append(member.filename)
    if f"lib/{abi}/libmain.so" not in verified:
        raise ValueError("APK does not contain the actual Dioxus native app")
    signing = subprocess.check_output([str(sdk / "build-tools/35.0.0/apksigner"), "verify", "--verbose", "--print-certs", str(apk)], text=True)
    if "Android Debug" not in signing:
        raise ValueError("unexpected signing identity for an engineering APK")
    return {"abi": abi, "native_libraries": verified, "16k_elf_alignment": True,
            "signing": "Android debug certificate; not a production signature", "permissions": sorted(permissions)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", required=True, choices=TARGETS)
    args = parser.parse_args()
    output = ROOT / "artifacts/android" / args.target
    output.mkdir(parents=True, exist_ok=True)
    report = {"target": args.target, "build_verified": False, "runtime_verified": False, "production_ready": False,
              "feature_scope": "SAF read-only selection/browsing/UTF-8 preview; no Android mutation controls"}
    tail = collections.deque(maxlen=120)
    code = 1
    try:
        command = ["dx", "build", "--platform", "android", "--target", args.target, "--release", "--no-default-features", "--features", "mobile", "--cargo-args=--locked", "--rustc-args=-Clink-arg=-Wl,-z,max-page-size=16384"]
        report["command"] = command
        stop = "android-" + uuid.uuid4().hex
        github = os.environ.get("GITHUB_ACTIONS") == "true"
        if github:
            print("::stop-commands::" + stop, flush=True)
        with subprocess.Popen(command, cwd=APP, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, encoding="utf-8", errors="replace") as process:
            while True:
                line = process.stdout.readline(1_048_577)
                if not line:
                    break
                if len(line) > 1_048_576:
                    process.kill()
                    raise ValueError("Android diagnostic line exceeds resource limit")
                print(line, end="", flush=True)
                tail.append(re.sub(r"\x1b\[[0-9;]*[A-Za-z]", "", line)[-4000:])
            code = process.wait()
        if github:
            print("::" + stop + "::", flush=True)
        if code:
            raise ValueError("dx Android build failed (see diagnostic tail)")
        candidates = list((APP / "target/dx").rglob("app-debug.apk"))
        if len(candidates) != 1:
            raise ValueError(f"expected exactly one debug APK; found {len(candidates)}")
        metadata = verify_apk(candidates[0], args.target, pathlib.Path(os.environ["ANDROID_HOME"]))
        dest = output / f"aurex-0.1.0-{TARGETS[args.target][0]}-debug.apk"
        shutil.copyfile(candidates[0], dest)
        digest = hashlib.sha256(dest.read_bytes()).hexdigest()
        (output / (dest.name + ".sha256")).write_text(f"{digest}  {dest.name}\n")
        report.update(metadata, apk=dest.name, sha256=digest, build_verified=True,
                      git_commit=subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip())
        code = 0
    except (OSError, ValueError, KeyError, subprocess.SubprocessError, zipfile.BadZipFile) as error:
        report["error"] = str(error)
        code = 1
    report["diagnostic_tail"] = "".join(tail)[-20000:]
    (output / "build-metadata.json").write_text(json.dumps(report, indent=2) + "\n")
    if code and os.environ.get("GITHUB_ACTIONS") == "true":
        excerpt = (report.get("error", "Android build failed") + "\n" + report["diagnostic_tail"])[-22000:]
        print("::error title=Android build or verification failed::" + excerpt.replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A"), flush=True)
    raise SystemExit(code)


if __name__ == "__main__":
    main()
