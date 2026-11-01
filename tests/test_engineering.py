"""Real artifact regression fixtures; never extract or execute untrusted archive members."""
import gzip
import io
import pathlib
import platform
import tarfile
import tempfile
import unittest
from unittest.mock import patch
from scripts import package, platform_gate


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
        self.assertEqual(report["state"], "BLOCKED")
        self.assertFalse(report["checks"]["android_storage_provider"])

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


if __name__ == "__main__":
    unittest.main()
