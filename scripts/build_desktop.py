#!/usr/bin/env python3
"""Native GUI compilation only, with API-readable errors; never a runtime claim."""
import collections
import json
import os
import pathlib
import subprocess
import uuid

ROOT = pathlib.Path(__file__).resolve().parents[1]


def escape(text):
    return text.replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A")


def main():
    command = ["cargo", "build", "--manifest-path", "apps/aurex-app/Cargo.toml", "--release", "--locked",
               "--message-format=json"]
    errors, tail = [], collections.deque(maxlen=60)
    github = os.environ.get("GITHUB_ACTIONS") == "true"
    stop = "desktop-" + uuid.uuid4().hex
    if github:
        print(f"::stop-commands::{stop}", flush=True)
    with subprocess.Popen(command, cwd=ROOT, text=True, encoding="utf-8", errors="replace", stdout=subprocess.PIPE, stderr=subprocess.STDOUT) as process:
        while True:
            line = process.stdout.readline(1_048_577)
            if not line:
                break
            if len(line) > 1_048_576:
                process.kill()
                errors.append("Compiler diagnostic exceeded the 1 MiB per-line resource limit")
                break
            try:
                record = json.loads(line)
            except json.JSONDecodeError:
                record = {}
            if not isinstance(record, dict):
                record = {}
            if record.get("reason") == "compiler-message":
                message = record["message"]
                rendered = message.get("rendered") or message["message"]
                print(rendered, flush=True)
                tail.append(rendered[-8000:])
                if message["level"] == "error" and len(errors) < 16:
                    errors.append(rendered[-8000:])
            elif not record:
                print(line, end="", flush=True)
                tail.append(line[-8000:])
        code = process.wait()
    if github:
        print(f"::{stop}::", flush=True)
    if code and not errors:
        errors.append("".join(tail)[-8000:])
    if github and code:
        for error in errors:
            print("::error title=Native GUI compilation failed::" + escape(error), flush=True)
    path = ROOT / "artifacts/desktop/compilation.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps({"command": command, "exit_code": code, "errors": errors,
                                "diagnostic_tail": "".join(tail)[-16000:], "runtime_verified": False}, indent=2) + "\n")
    raise SystemExit(0 if code == 0 else 1)


if __name__ == "__main__":
    main()
