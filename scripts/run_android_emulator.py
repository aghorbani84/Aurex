#!/usr/bin/env python3
"""Own a short-lived disposable CI emulator, wait boundedly, and always terminate it."""
import argparse
import json
import os
import pathlib
import subprocess
import sys
import tempfile
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--prepare-ci", action="store_true", required=True)
    parser.parse_args()
    if os.environ.get("GITHUB_ACTIONS") != "true":
        raise RuntimeError("Emulator setup/wipe is restricted to disposable GitHub runners")
    sdk = pathlib.Path(os.environ["ANDROID_HOME"])
    with tempfile.TemporaryDirectory(prefix="aurex-ci-avd-", dir=os.environ["RUNNER_TEMP"]) as owned:
        # avdmanager and emulator releases can disagree about default HOME locations.
        # Give both explicit, identical, private paths; never wipe an existing AVD.
        os.environ["ANDROID_USER_HOME"] = owned
        os.environ["ANDROID_EMULATOR_HOME"] = owned
        os.environ["ANDROID_AVD_HOME"] = str(pathlib.Path(owned) / "avd")
        pathlib.Path(os.environ["ANDROID_AVD_HOME"]).mkdir()
        os.environ["ANDROID_SERIAL"] = "emulator-5554"
        os.environ["PATH"] = str(sdk / "platform-tools") + os.pathsep + os.environ.get("PATH", "")
        run_owned(sdk, pathlib.Path(owned))


def run_owned(sdk, owned):
    log_path = pathlib.Path("artifacts/android/x86_64-linux-android/emulator.log")
    log_path.parent.mkdir(parents=True, exist_ok=True)
    for command, answer in [
        ([str(sdk / "cmdline-tools/latest/bin/sdkmanager"), "platform-tools", "emulator", "system-images;android-35;google_apis;x86_64"], None),
        ([str(sdk / "cmdline-tools/latest/bin/avdmanager"), "create", "avd", "--name", "aurex-ci", "--path", str(owned / "avd/aurex-ci.avd"), "--package", "system-images;android-35;google_apis;x86_64", "--device", "pixel_4"], "no\n"),
        (["sudo", "setfacl", "-m", "u:" + os.environ["USER"] + ":rw", "/dev/kvm"], None),
    ]:
        result = subprocess.run(command, input=answer, text=True, capture_output=True, timeout=300)
        if result.returncode:
            raise RuntimeError("CI emulator preparation failed: " + " ".join(command) + "\n" + (result.stdout + result.stderr)[-10000:])
    if not (pathlib.Path(os.environ["ANDROID_AVD_HOME"]) / "aurex-ci.ini").is_file():
        raise RuntimeError("avdmanager did not create the owned AVD descriptor; " +
                           "; ".join(str(path) for path in owned.rglob("*.ini")))
    with log_path.open("w") as log:
        with subprocess.Popen([str(sdk / "emulator/emulator"), "-avd", "aurex-ci", "-port", "5554", "-no-window", "-no-audio", "-no-boot-anim", "-no-snapshot", "-wipe-data", "-gpu", "swiftshader_indirect", "-memory", "2048", "-cores", "2"], stdout=log, stderr=subprocess.STDOUT) as emulator:
            try:
                deadline = time.monotonic() + 180
                while time.monotonic() < deadline:
                    if emulator.poll() is not None:
                        raise RuntimeError("emulator exited before boot; inspect emulator.log")
                    result = subprocess.run(["adb", "shell", "getprop", "sys.boot_completed"], capture_output=True, text=True, timeout=10)
                    if result.returncode == 0 and result.stdout.strip() == "1":
                        break
                    time.sleep(2)
                else:
                    raise RuntimeError("emulator boot deadline exceeded")
                subprocess.run(["adb", "shell", "input", "keyevent", "82"], check=True, timeout=10)
                subprocess.run([sys.executable, "scripts/android_smoke.py"], check=True, timeout=300)
            finally:
                emulator.terminate()
                try:
                    emulator.wait(timeout=15)
                except subprocess.TimeoutExpired:
                    emulator.kill()
                    emulator.wait()


if __name__ == "__main__":
    try:
        main()
    except (OSError, KeyError, RuntimeError, subprocess.SubprocessError) as error:
        path = pathlib.Path("artifacts/android/x86_64-linux-android")
        path.mkdir(parents=True, exist_ok=True)
        diagnostic = str(error)
        log = path / "emulator.log"
        if log.is_file():
            with log.open("rb") as stream:
                stream.seek(max(0, log.stat().st_size - 8000))
                diagnostic += "\n" + stream.read(8000).decode("utf-8", errors="replace")
        (path / "emulator-failure.json").write_text(json.dumps({"passed": False, "error": diagnostic}, indent=2) + "\n")
        if os.environ.get("GITHUB_ACTIONS") == "true":
            print("::error title=Android emulator acceptance failed::" + diagnostic[-18000:].replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A"))
        else:
            print(diagnostic, file=sys.stderr)
        raise SystemExit(1)
