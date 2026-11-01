"""Artifact parser unit fixtures, not Android runtime or signing certification."""
import pathlib
import struct
import tempfile
import unittest
import zipfile
from unittest.mock import patch
from scripts import build_android


def elf(machine=183, alignment=16384):
    data = bytearray(120)
    data[:6] = b"\x7fELF\x02\x01"
    struct.pack_into("<H", data, 18, machine)
    struct.pack_into("<Q", data, 32, 64)
    struct.pack_into("<HH", data, 54, 56, 1)
    struct.pack_into("<IIQQQQQQ", data, 64, 1, 5, 0, 0, 0, 120, 120, alignment)
    return bytes(data)


class AndroidArtifactSafety(unittest.TestCase):
    def test_real_elf_header_architecture_and_16k_requirement(self):
        self.assertEqual(build_android.elf_alignment(elf(), 183), 1)
        for data in [elf(62), elf(alignment=4096), elf(alignment=20000), b"not ELF", elf()[:80]]:
            with self.assertRaises(ValueError):
                build_android.elf_alignment(data, 183)

    def test_invalid_load_segment_bounds_and_congruence(self):
        for field, value in [(5, 121), (6, 119), (2, 1), (3, 1)]:
            data = bytearray(elf())
            header = list(struct.unpack_from("<IIQQQQQQ", data, 64))
            header[field] = value
            struct.pack_into("<IIQQQQQQ", data, 64, *header)
            with self.subTest(field=field), self.assertRaises(ValueError):
                build_android.elf_alignment(data, 183)

    def manifest(self, permission="", backup="false", debug="true", package="io.github.aghorbani84.aurex"):
        return f'''<manifest xmlns:android="http://schemas.android.com/apk/res/android" package="{package}">
        {permission}<application android:debuggable="{debug}" android:allowBackup="{backup}" android:usesCleartextTraffic="false" /></manifest>'''

    def verify(self, manifest, library="lib/arm64-v8a/libmain.so"):
        with tempfile.TemporaryDirectory() as folder:
            path = pathlib.Path(folder) / "fixture.apk"
            with zipfile.ZipFile(path, "w") as archive:
                archive.writestr(library, elf())
            def tool(command, **kwargs):
                return manifest if command[0].endswith("apkanalyzer") else "Signer certificate DN: CN=Android Debug"
            with patch.object(build_android.subprocess, "check_output", side_effect=tool):
                return build_android.verify_apk(path, "aarch64-linux-android", pathlib.Path(folder))

    def test_debug_scope_does_not_imply_production_approval(self):
        report = self.verify(self.manifest())
        self.assertEqual(report["abi"], "arm64-v8a")
        self.assertIn("not a production signature", report["signing"])

    def test_wrong_identity_backup_and_release_label_are_refused(self):
        for options in [{"package": "com.example.demo"}, {"backup": "true"}, {"debug": "false"}]:
            with self.subTest(options=options), self.assertRaises(ValueError):
                self.verify(self.manifest(**options))

    def test_broad_storage_permissions_are_refused(self):
        with self.assertRaisesRegex(ValueError, "broad storage"):
            self.verify(self.manifest('<uses-permission android:name="android.permission.MANAGE_EXTERNAL_STORAGE" />'))

    def test_wrong_abi_is_refused_even_if_manifest_looks_valid(self):
        with self.assertRaisesRegex(ValueError, "unexpected native ABI"):
            self.verify(self.manifest(), "lib/x86_64/libmain.so")
