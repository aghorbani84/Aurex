#!/usr/bin/env python3
"""Own a short-lived disposable CI emulator, wait boundedly, and always terminate it."""
import os
import pathlib
import subprocess
import sys
import time


def main():
    sdk = pathlib.Path(os.environ["ANDROID_HOME"])
    log_path = pathlib.Path("artifacts/android/x86_64-linux-android/emulator.log")
    log_path.parent.mkdir(parents=True, exist_ok=True)
    with log_path.open("w") as log:
        with subprocess.Popen([str(sdk / "emulator/emulator"), "-avd", "aurex-ci", "-no-window", "-no-audio", "-no-boot-anim", "-no-snapshot", "-wipe-data", "-gpu", "swiftshader_indirect", "-memory", "2048", "-cores", "2"], stdout=log, stderr=subprocess.STDOUT) as emulator:
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
    main()
