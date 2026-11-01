"""Real artifact regression fixtures; never extract or execute untrusted archive members."""
import gzip
import io
import json
import os
import contextlib
from types import SimpleNamespace
import pathlib
import platform
import tarfile
import tempfile
import unittest
from unittest.mock import MagicMock, patch
from scripts import package, platform_gate, audit_dependencies, build_desktop


class ArchiveSafety(unittest.TestCase):
    def fixture(self, folder, names):
        path = pathlib.Path(folder) / "crafted.tar.gz"
        with tarfile.open(path, "w:gz") as archive:
            for name, kind in names:
                member = tarfile.TarInfo(name)
                member.type = kind
                member.size = 1 if kind == tarfile.REGTYPE else 0
                archive.addfile(member, io.BytesIO(b"x"))
        return path

    def test_traversal_absolute_links_and_duplicate_names(self):
        for entries in [[("../escape", tarfile.REGTYPE)], [("/escape", tarfile.REGTYPE)],
                        [("aurex-cli-test/link", tarfile.SYMTYPE)],
                        [("aurex-cli-test/x", tarfile.REGTYPE)] * 2]:
            with self.subTest(entries=entries), tempfile.TemporaryDirectory() as root:
                with self.assertRaises(ValueError):
                    package.verify(self.fixture(root, entries))
                self.assertFalse((pathlib.Path(root) / "escape").exists())

    def test_oversized_pax_header_refused_before_allocation(self):
        with tempfile.TemporaryDirectory() as root:
            path = pathlib.Path(root) / "pax.tar.gz"
            header = tarfile.TarInfo("pax")
            header.type, header.size = tarfile.XHDTYPE, package.MAX_MEMBER + 1
            with gzip.open(path, "wb") as stream:
                stream.write(header.tobuf() + b"\0" * 1024)
            with self.assertRaisesRegex(ValueError, "oversized tar"):
                package.verify(path)

    def test_decompression_bomb_limit_is_enforced(self):
        with tempfile.TemporaryDirectory() as root:
            path = pathlib.Path(root) / "bomb.tar.gz"
            with gzip.open(path, "wb") as stream:
                stream.write(b"\0" * (2 * 1024 * 1024))
            with patch.object(package, "MAX_TOTAL", 65536), self.assertRaisesRegex(ValueError, "decompression limit"):
                package.verify(path)

    def test_corrupt_checksum_manifest_is_refused_without_execution(self):
        with tempfile.TemporaryDirectory() as root:
            path = pathlib.Path(root) / "tampered.tar.gz"
            with tarfile.open(path, "w:gz") as archive:
                for name, data in [("payload", b"x"), ("MANIFEST.sha256", b"0" * 64 + b"  payload\n")]:
                    info = tarfile.TarInfo("aurex-cli-test/" + name)
                    info.size = len(data)
                    archive.addfile(info, io.BytesIO(data))
            with self.assertRaisesRegex(ValueError, "checksum mismatch"):
                package.verify(path)

    def test_android_cannot_misreport_packaging_as_verified(self):
        report = platform_gate.inspect_android()
        self.assertFalse(report["packaging_verified"])
        self.assertIn(report["state"], ["BLOCKED", "PREREQUISITES_ONLY"])
        self.assertTrue(report["checks"]["android_storage_provider"])

    @unittest.skipUnless(platform.system() == "Linux" and platform.machine() == "x86_64", "validated package target only")
    def test_actual_candidate_is_reproducible_and_launches(self):
        if not (package.ROOT / "target/release/aurex-cli").is_file():
            self.skipTest("build the release CLI first")
        with tempfile.TemporaryDirectory() as root:
            a = package.build(pathlib.Path(root) / "a", allow_dirty=True)
            b = package.build(pathlib.Path(root) / "b", allow_dirty=True)
            self.assertEqual(a.read_bytes(), b.read_bytes())
            self.assertFalse(package.verify(a)["production_release"])
            self.assertEqual(package.verify(a)["target"], package.TARGET)


class AuditFailurePolicy(unittest.TestCase):
    def test_failure_does_not_skip_other_graphs_or_become_a_pass(self):
        calls = []
        def runner(command, **kwargs):
            calls.append(command)
            return SimpleNamespace(returncode=1 if len(calls) == 1 else 0, stdout="fixture diagnostic", stderr="")
        with contextlib.redirect_stdout(io.StringIO()):
            results = audit_dependencies.collect(runner)
        self.assertEqual(len(calls), 4)
        self.assertFalse(all(r["exit_code"] == 0 for r in results))
        self.assertEqual(results[-1]["check"], "desktop-deny")

    def test_missing_tools_remain_failed_checks(self):
        def missing(*args, **kwargs):
            raise FileNotFoundError("controlled missing tool")
        with contextlib.redirect_stdout(io.StringIO()):
            results = audit_dependencies.collect(missing)
        self.assertEqual(len(results), 4)
        self.assertTrue(all(r["exit_code"] != 0 for r in results))


class DesktopDiagnosticPolicy(unittest.TestCase):
    def run_fixture(self, text, returncode):
        with tempfile.TemporaryDirectory() as temp:
            process = MagicMock()
            process.stdout = io.StringIO(text)
            process.wait.return_value = returncode
            factory = MagicMock()
            factory.__enter__.return_value = process
            output = io.StringIO()
            with patch.object(build_desktop, "ROOT", pathlib.Path(temp)), \
                 patch.object(build_desktop.subprocess, "Popen", return_value=factory), \
                 patch.dict(os.environ, {"GITHUB_ACTIONS": "true"}), contextlib.redirect_stdout(output):
                with self.assertRaises(SystemExit) as exit:
                    build_desktop.main()
            report = json.loads((pathlib.Path(temp) / "artifacts/desktop/compilation.json").read_text())
            self.assertEqual(exit.exception.code, 1)
            self.assertFalse(report["runtime_verified"])
            return process, report, output.getvalue()

    def test_compiler_failure_and_json_scalar_are_not_lost_or_executed(self):
        text = "true\n" + json.dumps({"reason": "compiler-message", "message": {
            "rendered": "fixture %\n::warning::not a command", "level": "error"}}) + "\n"
        process, report, output = self.run_fixture(text, 101)
        self.assertEqual(report["exit_code"], 101)
        self.assertIn("fixture %25%0A::warning::not a command", output)
        self.assertIn("::stop-commands::desktop-", output)
        process.kill.assert_not_called()

    def test_oversized_diagnostic_fails_closed(self):
        process, report, output = self.run_fixture("x" * 1_048_577, -9)
        process.kill.assert_called_once()
        self.assertIn("resource limit", report["errors"][0])


if __name__ == "__main__":
    unittest.main()
