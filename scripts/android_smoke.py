#!/usr/bin/env python3
"""Real system-picker/UI test on a disposable Android emulator only (never a user's device)."""
import hashlib
import json
import pathlib
import re
import subprocess
import tempfile
import time
import uuid
import xml.etree.ElementTree as ET

ROOT = pathlib.Path(__file__).resolve().parents[1]
PACKAGE = "io.github.aghorbani84.aurex"


def adb(*arguments, binary=False):
    return subprocess.check_output(["adb", *arguments], timeout=20, text=not binary)


def hierarchy():
    adb("shell", "uiautomator", "dump", "--compressed", "/sdcard/aurex-ci-window.xml")
    return ET.fromstring(adb("shell", "cat", "/sdcard/aurex-ci-window.xml"))


def matching(tree, text, contains=False):
    expected = text.casefold()
    for node in tree.iter("node"):
        values = [node.get("text", ""), node.get("content-desc", "")]
        if any(expected in value.casefold() if contains else expected == value.casefold() for value in values):
            if node.get("enabled") == "true" and node.get("bounds") not in [None, "[0,0][0,0]"]:
                return node
    return None


def wait(text, contains=False, timeout=45):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        tree = hierarchy()
        node = matching(tree, text, contains)
        if node is not None:
            return node
        time.sleep(0.5)
    raise ValueError("Android UI assertion timed out: " + text)


def tap(text):
    node = wait(text)
    values = re.fullmatch(r"\[(\d+),(\d+)\]\[(\d+),(\d+)\]", node.get("bounds", ""))
    if not values:
        raise ValueError("invalid UI bounds")
    x1, y1, x2, y2 = map(int, values.groups())
    adb("shell", "input", "tap", str((x1+x2)//2), str((y1+y2)//2))


def start():
    adb("shell", "am", "start", "-W", "-n", PACKAGE + "/dev.dioxus.main.MainActivity")


def main():
    output = ROOT / "artifacts/android/x86_64-linux-android"
    output.mkdir(parents=True, exist_ok=True)
    report = {"passed": False, "production_ready": False, "scope": "Real emulator system picker/cancel, listing, UTF-8 preview, persisted grant restart and forget; not all-provider/device certification"}
    fixture = "/sdcard/Documents/Aurex-Smoke-" + uuid.uuid4().hex
    created = False
    safe_device = False
    try:
        if adb("shell", "getprop", "ro.kernel.qemu").strip() != "1":
            raise ValueError("Refusing test: this is not a disposable emulator")
        safe_device = True
        report["api"] = adb("shell", "getprop", "ro.build.version.sdk").strip()
        report["abi"] = adb("shell", "getprop", "ro.product.cpu.abi").strip()
        apks = list(output.glob("*-debug.apk"))
        if len(apks) != 1:
            raise ValueError("expected one verified engineering APK")
        adb("install", "-r", str(apks[0]))
        payload = b"AUREX Android SAF runtime verification\n<script>not executed</script>\n"
        with tempfile.TemporaryDirectory(prefix="aurex-android-fixture-") as folder:
            path = pathlib.Path(folder) / "aurex-smoke.txt"
            path.write_bytes(payload)
            adb("shell", "mkdir", "-p", fixture)
            created = True
            adb("push", str(path), fixture + "/aurex-smoke.txt")
        start()
        tap("Choose a folder")
        tap(fixture.rsplit("/", 1)[-1])
        tap("Use this folder")
        tap("Allow")
        tap("aurex-smoke.txt")
        wait("AUREX Android SAF runtime verification", contains=True)
        wait("<script>not executed</script>", contains=True)
        tap("Close preview")
        adb("shell", "am", "force-stop", PACKAGE)
        start()
        wait("aurex-smoke.txt")
        report["persisted_after_process_restart"] = True
        tap("Forget permission")
        wait("Folder permission forgotten. Files were not deleted.")
        adb("shell", "am", "force-stop", PACKAGE)
        start()
        tap("Choose a folder")
        adb("shell", "input", "keyevent", "4")
        wait("Folder selection cancelled")
        if matching(hierarchy(), "aurex-smoke.txt") is not None:
            raise ValueError("forgotten/cancelled grant still exposes document rows")
        actual = adb("exec-out", "cat", fixture + "/aurex-smoke.txt", binary=True)
        if hashlib.sha256(actual).digest() != hashlib.sha256(payload).digest():
            raise ValueError("read-only Android fixture changed")
        (output / "runtime.png").write_bytes(adb("exec-out", "screencap", "-p", binary=True))
        report["passed"] = True
    except (OSError, ValueError, subprocess.SubprocessError, ET.ParseError) as error:
        report["error"] = str(error)
        if safe_device:
            try:
                tree = hierarchy()
                report["visible_text"] = [n.get("text") or n.get("content-desc") for n in tree.iter("node") if n.get("text") or n.get("content-desc")][:80]
            except (OSError, subprocess.SubprocessError, ET.ParseError):
                report["visible_text"] = ["UI hierarchy unavailable"]
    finally:
        if created:
            # Exact owned UUID directory on an emulator we already checked. No wildcard cleanup.
            try:
                cleanup = subprocess.run(["adb", "shell", "rm", "-r", "--", fixture], timeout=20)
                if cleanup.returncode:
                    report["passed"] = False
                    report["cleanup_error"] = "Emulator fixture cleanup failed"
            except (OSError, subprocess.SubprocessError) as error:
                report["passed"] = False
                report["cleanup_error"] = str(error)
        (output / "runtime.json").write_text(json.dumps(report, indent=2) + "\n")
    if not report["passed"]:
        detail = json.dumps({k: report[k] for k in ["error", "cleanup_error", "visible_text"] if k in report})[:12000]
        print("::error title=Android SAF runtime test failed::" + detail.replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A"))
    print("PASS: real Android SAF emulator fixture" if report["passed"] else "FAIL: Android SAF emulator fixture; see runtime.json")
    raise SystemExit(0 if report["passed"] else 1)


if __name__ == "__main__":
    main()
