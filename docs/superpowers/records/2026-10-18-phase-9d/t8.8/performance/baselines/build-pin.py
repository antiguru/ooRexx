#!/usr/bin/env python3
"""Build one committed release baseline and preserve its provenance."""

import datetime
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time


def output(command, cwd):
    return subprocess.check_output(command, cwd=cwd, text=True).strip()


label, worktree_name, expected_revision, pin_name = sys.argv[1:]
worktree = Path(worktree_name)
records = Path(__file__).resolve().parent
target = Path("/tmp/codex-p9d-t88-perf-target")
pin = Path("/tmp/codex-p9d-t88-pins") / pin_name
status_path = records / f"{label}.json"
stdout_path = records / f"{label}.stdout"
stderr_path = records / f"{label}.stderr"
if any(path.exists() for path in (pin, status_path, stdout_path, stderr_path)):
    raise SystemExit("Refusing to overwrite an existing baseline artifact")

revision = output(["git", "rev-parse", "HEAD"], worktree)
if revision != expected_revision:
    raise SystemExit(f"Unexpected revision: {revision}")
source_status = output(["git", "status", "--porcelain"], worktree)
if source_status:
    raise SystemExit(f"Source worktree is dirty: {source_status}")
free_before = shutil.disk_usage("/tmp").free
if free_before < 4 * 1024**3:
    raise SystemExit(f"Insufficient build headroom: {free_before} bytes free")

command = ["cargo", "build", "-j4", "--release", "-p", "rexx-exec", "--bin", "rexx"]
environment = dict(os.environ)
environment.update(CARGO_INCREMENTAL="0", CARGO_TARGET_DIR=str(target))
cwd = worktree / "rust"
record = {
    "label": label,
    "revision": revision,
    "source_tree": output(["git", "rev-parse", "HEAD^{tree}"], worktree),
    "rust_source_tree": output(["git", "rev-parse", "HEAD:rust"], worktree),
    "source_status_before": source_status,
    "branch": output(["git", "branch", "--show-current"], worktree),
    "worktree": str(worktree),
    "build_cwd": str(cwd),
    "build_command": command,
    "driver_command": ["memcap", "8G", "python3", str(Path(__file__).resolve()), *sys.argv[1:]],
    "environment": {key: value for key, value in sorted(environment.items())
                    if key.startswith(("CARGO_", "RUST", "REXX_"))},
    "rustc_version": output(["rustc", "--version", "--verbose"], cwd),
    "cargo_version": output(["cargo", "--version", "--verbose"], cwd),
    "memory_cap": "8G",
    "deadline_seconds": 530,
    "stdout": str(stdout_path),
    "stderr": str(stderr_path),
    "tmp_free_bytes_before": free_before,
    "started_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
}
started = time.monotonic()
with stdout_path.open("xb") as stdout, stderr_path.open("xb") as stderr:
    process = subprocess.Popen(command, cwd=cwd, env=environment,
                               stdout=stdout, stderr=stderr, start_new_session=True)
    try:
        status = process.wait(timeout=530)
        record["timed_out"] = False
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        status = process.wait()
        record["timed_out"] = True
record.update(
    build_status=status,
    elapsed_seconds=time.monotonic() - started,
    finished_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
    source_status_after=output(["git", "status", "--porcelain"], worktree),
    revision_after=output(["git", "rev-parse", "HEAD"], worktree),
    tmp_free_bytes_after=shutil.disk_usage("/tmp").free,
)
if status == 0:
    if record["source_status_after"] or record["revision_after"] != revision:
        raise SystemExit("Build changed the source worktree")
    with (target / "release/rexx").open("rb") as source, pin.open("xb") as destination:
        shutil.copyfileobj(source, destination)
    shutil.copymode(target / "release/rexx", pin)
    record.update(binary=str(pin), binary_size_bytes=pin.stat().st_size,
                  binary_sha256=hashlib.file_digest(pin.open("rb"), "sha256").hexdigest())
status_path.write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps(record, indent=2))
raise SystemExit(status if status >= 0 else 128 - status)
