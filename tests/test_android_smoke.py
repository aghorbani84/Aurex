"""Harness safety regressions only; these do not establish Android runtime support."""
import contextlib
import io
import json
import pathlib
import tempfile
import unittest
from unittest.mock import patch
from scripts import android_smoke, run_android_emulator


class EmulatorHarnessSafety(unittest.TestCase):
    def test_refuses_physical_devices_before_install_ui_or_fixture_operations(self):
        with tempfile.TemporaryDirectory() as folder:
            with patch.object(android_smoke, "ROOT", pathlib.Path(folder)), patch.object(
                android_smoke, "adb", return_value="0\n"
            ) as adb, contextlib.redirect_stdout(io.StringIO()), self.assertRaises(SystemExit) as failure:
                android_smoke.main()
            self.assertEqual(failure.exception.code, 1)
            adb.assert_called_once_with("shell", "getprop", "ro.kernel.qemu")
            report = json.loads((pathlib.Path(folder) / "artifacts/android/x86_64-linux-android/runtime.json").read_text())
            self.assertFalse(report["passed"])
            self.assertFalse(report["production_ready"])
            self.assertIn("Refusing test", report["error"])

    def test_ui_match_requires_enabled_nonempty_bounds(self):
        tree = android_smoke.ET.fromstring('<hierarchy><node text="Allow" enabled="false" bounds="[0,0][50,50]"/><node text="Allow" enabled="true" bounds="[0,0][0,0]"/></hierarchy>')
        self.assertIsNone(android_smoke.matching(tree, "Allow"))


class EmulatorOwnerSafety(unittest.TestCase):
    def test_refuses_non_ci_hosts_before_installing_or_wiping_any_avd(self):
        with patch.object(run_android_emulator.sys, "argv", ["runner", "--prepare-ci"]), patch.dict(
            run_android_emulator.os.environ, {}, clear=True
        ), patch.object(run_android_emulator.subprocess, "run") as commands:
            with self.assertRaisesRegex(RuntimeError, "restricted"):
                run_android_emulator.main()
            commands.assert_not_called()
