#!/usr/bin/env python3
"""Install one checksum-verified official dx build tool into an explicit user directory."""
import argparse
import hashlib
import pathlib
import shutil
import tarfile
import tempfile
import urllib.request

URL = "https://github.com/DioxusLabs/dioxus/releases/download/v0.7.10/dx-x86_64-unknown-linux-gnu.tar.gz"
SHA256 = "4363e4ed2a3f1eb7f4d38d2d59aed59ce43271c44c16b425e92c89a64761fbe7"


def install(output):
    output = pathlib.Path(output)
    output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryFile() as archive:
        checksum = hashlib.sha256()
        size = 0
        with urllib.request.urlopen(URL, timeout=90) as response:
            while block := response.read(64 * 1024):
                size += len(block)
                if size > 256 * 1024 * 1024:
                    raise ValueError("dx download exceeds limit")
                checksum.update(block)
                archive.write(block)
        if checksum.hexdigest() != SHA256:
            raise ValueError("official dx asset checksum mismatch")
        archive.seek(0)
        with tarfile.open(fileobj=archive, mode="r:gz") as tar:
            matches = [m for m in tar.getmembers() if m.isfile() and pathlib.PurePosixPath(m.name).name == "dx"]
            if len(matches) != 1 or matches[0].size > 256 * 1024 * 1024:
                raise ValueError("unexpected dx asset layout")
            # Copy only this verified regular member; never extract archive paths/links.
            with tar.extractfile(matches[0]) as source, (output / "dx").open("wb") as dest:
                shutil.copyfileobj(source, dest, 64 * 1024)
    (output / "dx").chmod(0o700)
    print(f"Verified dx 0.7.10: {SHA256}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True)
    install(parser.parse_args().output)
