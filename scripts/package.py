#!/usr/bin/env python3
"""Build/verify a reproducible, unsigned Linux CLI engineering archive; never publish.

The GUI and other providers are not silently substituted with a CLI release. This
archive is explicitly labelled CLI-only and includes source/toolchain provenance.
"""
import argparse
import gzip
import hashlib
import io
import json
import pathlib
import platform
import re
import subprocess
import tarfile
import tempfile
import tomllib
import uuid

ROOT = pathlib.Path(__file__).resolve().parents[1]
TARGET = "x86_64-unknown-linux-gnu"
MAX_MEMBER = 64 * 1024 * 1024
MAX_TOTAL = 128 * 1024 * 1024


def run(*args, cwd=ROOT):
    return subprocess.check_output(args, cwd=cwd, text=True).strip()


def digest(data):
    return hashlib.sha256(data).hexdigest()


def json_bytes(value):
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()


def smoke(binary):
    """Execute only our freshly built CLI, against an exclusively owned temp root."""
    help_text = run(str(binary), "--help")
    if "root-scoped" not in help_text:
        raise ValueError("artifact is not the expected AUREX CLI")
    with tempfile.TemporaryDirectory(prefix="aurex-package-smoke-") as temp:
        root = pathlib.Path(temp)
        (root / "source.txt").write_bytes(b"AUREX artifact smoke\n")
        for args in [("list",), ("--write", "mkdir", "folder"),
                     ("--write", "copy", "source.txt", "folder/copy.txt"),
                     ("--write", "move", "folder/copy.txt", "moved.txt")]:
            run(str(binary), temp, *args)
        if (root / "moved.txt").read_bytes() != (root / "source.txt").read_bytes():
            raise ValueError("artifact transfer smoke corrupted data")
        conflict = subprocess.run([str(binary), temp, "--write", "copy", "source.txt", "moved.txt"], capture_output=True)
        if conflict.returncode == 0:
            raise ValueError("artifact overwrote an existing destination")
        run(str(binary), temp, "--write", "delete", "moved.txt", "--confirm")
        if not (root / "source.txt").is_file():
            raise ValueError("artifact destroyed source")


def build(output, allow_dirty=False):
    if platform.system() != "Linux" or platform.machine() != "x86_64":
        raise ValueError("packaging is validated only for Linux x86-64 CLI; GUI/Windows/Android blocked")
    dirty = bool(run("git", "status", "--porcelain"))
    if dirty and not allow_dirty:
        raise ValueError("refusing a dirty-tree candidate; use --allow-dirty only for labelled local verification")
    version = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["package"]["version"]
    binary = ROOT / "target" / "release" / "aurex-cli"
    if not binary.is_file() or binary.is_symlink():
        raise ValueError("build the release CLI first: cargo build --workspace --release --locked")
    smoke(binary)
    linking = run("ldd", str(binary))
    if "not found" in linking:
        raise ValueError("native runtime dependency missing")
    linking = re.sub(r"0x[0-9a-fA-F]+", "<runtime-address>", linking)
    commit = run("git", "rev-parse", "HEAD")
    epoch = int(run("git", "show", "-s", "--format=%ct", "HEAD"))
    prefix = f"aurex-cli-{version}-{TARGET}"
    metadata = {"schema": 1, "product": "AUREX CLI engineering candidate", "version": version,
                "target": TARGET, "git_commit": commit, "dirty": dirty, "source_date_epoch": epoch,
                "rustc": run("rustc", "--version"), "cargo": run("cargo", "--version"),
                "signing": "unsigned", "runtime_smoke": "PASS: CLI fixture operations only",
                "gui_runtime": "NOT TESTED; GUI is not in this archive", "production_release": False}
    files = {"bin/aurex-cli": binary.read_bytes(), "build-metadata.json": json_bytes(metadata),
             "native-dependencies.txt": (linking + "\n").encode()}
    for name in ["LICENSE", "README.md", "SECURITY.md", "Cargo.lock", "rust-toolchain.toml"]:
        files[name] = (ROOT / name).read_bytes()
    for name in ["FILE_OPERATIONS.md", "PLATFORM_SUPPORT.md", "RELEASES.md"]:
        files["docs/" + name] = (ROOT / "docs" / name).read_bytes()
    # Include actual notices from this toolchain, not invented licenses for native code.
    sysroot = pathlib.Path(run("rustc", "--print", "sysroot"))
    notice_root = sysroot / "share/doc/rust"
    if not (notice_root / "COPYRIGHT.html").is_file():
        raise ValueError("Rust toolchain notices missing; refuse an incomplete candidate")
    for path in sorted(notice_root.rglob("*")):
        if path.is_file() and not path.is_symlink() and (path.name.startswith("COPYRIGHT") or path.parent.name == "licenses"):
            files["notices/rust/" + path.relative_to(notice_root).as_posix()] = path.read_bytes()
    packages = tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]
    sbom = {"bomFormat": "CycloneDX", "specVersion": "1.5", "version": 1,
            "serialNumber": "urn:uuid:" + str(uuid.uuid5(uuid.NAMESPACE_URL, commit + digest(files["bin/aurex-cli"]))),
            "metadata": {"component": {"type": "application", "name": "aurex-cli", "version": version}},
            "components": [{"type": "library", "name": p["name"], "version": p["version"],
                            "licenses": [{"license": {"id": "MIT"}}]} for p in packages],
            "properties": [{"name": "aurex:inventory-scope", "value": "root Cargo.lock only; Rust std notices included; system runtime dependencies listed separately; not a complete binary composition or license audit"}]}
    files["sbom.cdx.json"] = json_bytes(sbom)
    files["MANIFEST.sha256"] = "".join(f"{digest(data)}  {name}\n" for name, data in sorted(files.items())).encode()
    output = pathlib.Path(output)
    output.mkdir(parents=True, exist_ok=True)
    archive = output / (prefix + ".tar.gz")
    # Explicit timestamp/ownership/order/gzip header avoids build-clock and host-owner drift.
    with archive.open("wb") as stream, gzip.GzipFile(fileobj=stream, filename="", mode="wb", mtime=epoch) as zipped:
        with tarfile.open(fileobj=zipped, mode="w") as tar:
            for name, data in sorted(files.items()):
                info = tarfile.TarInfo(prefix + "/" + name)
                info.size, info.mtime = len(data), epoch
                info.mode = 0o755 if name == "bin/aurex-cli" else 0o644
                tar.addfile(info, io.BytesIO(data))
    verify(archive, execute=True)
    checksum = digest(archive.read_bytes())
    (output / (archive.name + ".sha256")).write_text(f"{checksum}  {archive.name}\n")
    (output / "build-metadata.json").write_bytes(files["build-metadata.json"])
    (output / "sbom.cdx.json").write_bytes(files["sbom.cdx.json"])
    print(f"PASS {archive}: SHA-256 {checksum}; unsigned CLI only")
    return archive


def verify(archive, execute=False):
    """Bound and validate names/types/sizes/checksums before any optional fixture execution.

    No extractall, traversal, symlinks or archive-controlled executable paths.
    Execute is for trusted locally built packages only, not a malware sandbox.
    """
    archive = pathlib.Path(archive)
    if not archive.is_file() or archive.stat().st_size > MAX_TOTAL:
        raise ValueError("invalid or oversized compressed archive")
    # Bound decompression before tarfile parses PAX/GNU extension headers, which
    # otherwise can allocate based on attacker-controlled sizes before iteration.
    with tempfile.TemporaryFile() as expanded:
        with gzip.open(archive, "rb") as zipped:
            size = 0
            while chunk := zipped.read(64 * 1024):
                size += len(chunk)
                if size > MAX_TOTAL + 1024 * 1024:
                    raise ValueError("archive decompression limit exceeded")
                expanded.write(chunk)
        expanded.seek(0)
        class BoundedReader:
            def read(self, amount=-1):
                if amount < 0 or amount > MAX_MEMBER:
                    raise ValueError("oversized tar extension or read")
                return expanded.read(amount)
            def seek(self, offset, whence=0):
                position = expanded.seek(offset, whence)
                if not 0 <= position <= size:
                    raise ValueError("tar offset outside bounded input")
                return position
            def tell(self):
                return expanded.tell()
        with tarfile.open(fileobj=BoundedReader(), mode="r:") as tar:
            contents, total, prefix = {}, 0, None
            for index, member in enumerate(tar):
                path = pathlib.PurePosixPath(member.name)
                if index >= 100 or not member.isfile() or path.is_absolute() or ".." in path.parts or len(path.parts) < 2:
                    raise ValueError("unsafe archive member")
                prefix = prefix or path.parts[0]
                if prefix != path.parts[0] or not re.fullmatch(r"aurex-cli-[A-Za-z0-9._-]+", prefix):
                    raise ValueError("unexpected archive root")
                name = pathlib.PurePosixPath(*path.parts[1:]).as_posix()
                total += member.size
                if name in contents or not 0 <= member.size <= MAX_MEMBER or total > MAX_TOTAL:
                    raise ValueError("duplicate or oversized member")
                with tar.extractfile(member) as stream:
                    data = stream.read(MAX_MEMBER + 1)
                if len(data) != member.size:
                    raise ValueError("truncated/oversized archive member")
                contents[name] = data
    manifest = contents.pop("MANIFEST.sha256").decode()
    expected = {}
    for line in manifest.splitlines():
        checksum, name = line.split("  ", 1)
        if name in expected or not re.fullmatch(r"[0-9a-f]{64}", checksum):
            raise ValueError("invalid checksum manifest")
        expected[name] = checksum
    if expected != {name: digest(data) for name, data in contents.items()}:
        raise ValueError("archive checksum mismatch")
    metadata = json.loads(contents["build-metadata.json"])
    if metadata.get("target") != TARGET or metadata.get("production_release") is not False:
        raise ValueError("unexpected artifact identity")
    if execute:
        with tempfile.TemporaryDirectory(prefix="aurex-package-verify-") as temp:
            binary = pathlib.Path(temp) / "aurex-cli"
            binary.write_bytes(contents["bin/aurex-cli"])
            binary.chmod(0o700)
            smoke(binary)
    return metadata


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", default="artifacts/packages")
    parser.add_argument("--allow-dirty", action="store_true")
    parser.add_argument("--verify", type=pathlib.Path)
    args = parser.parse_args()
    if args.verify:
        print(json.dumps(verify(args.verify), indent=2))
    else:
        build(args.output, args.allow_dirty)


if __name__ == "__main__":
    main()
