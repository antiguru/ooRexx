#!/usr/bin/env python3
"""Retain numeric function attribution from existing arithmetic profiles."""

import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent
RAW = Path("/tmp/codex-p9d-t88-performance/base-ab-c")
ROUND = int(sys.argv[1])
OUT = ROOT / f"arith-attribution-r{ROUND}"
OUT.mkdir(exist_ok=True)
records = []
focused = []
for name in ("base", "ab"):
    profile = RAW / f"arith.{name}.r{ROUND}.cg"
    assert profile.exists()
    with profile.open("rb") as stream:
        profile_hash = hashlib.file_digest(stream, "sha256").hexdigest()
    for inclusive in ("no", "yes"):
        stem = f"{name}-inclusive-{inclusive}"
        command = ["callgrind_annotate", "--auto=no", "--show-percs=no", "--threshold=100",
                   f"--inclusive={inclusive}", str(profile)]
        started = time.monotonic()
        result = subprocess.run(command, capture_output=True, timeout=30)
        (OUT / f"{stem}.stdout").write_bytes(result.stdout)
        (OUT / f"{stem}.stderr").write_bytes(result.stderr)
        record = {"name": name, "round": ROUND, "inclusive": inclusive, "profile": str(profile),
                  "profile_sha256": profile_hash, "command": command, "status": result.returncode,
                  "elapsed_seconds": time.monotonic() - started,
                  "stdout": f"{stem}.stdout", "stderr": f"{stem}.stderr"}
        records.append(record)
        assert result.returncode == 0, record
        for line in result.stdout.decode().splitlines():
            if "rexx-num/src/pow.rs" in line or ("rexx_num" in line and re.search(r"pow|as_whole|whole_value|max_value_for_digits", line)):
                focused.append(f"{name}\tinclusive={inclusive}\t{line.strip()}")
    worktree = Path("/tmp/codex-p9d-t88-perf-base" if name == "base" else "/tmp/codex-p9d-t88-perf-a")
    sources = [worktree / "rust/crates/rexx-num/src" / filename for filename in ("pow.rs", "lib.rs", "digits.rs")]
    command = ["callgrind_annotate", "--auto=no", "--show-percs=no", "--threshold=100", "--context=2", str(profile), *map(str, sources)]
    started = time.monotonic()
    result = subprocess.run(command, capture_output=True, timeout=30)
    (OUT / f"{name}-source.stdout").write_bytes(result.stdout)
    (OUT / f"{name}-source.stderr").write_bytes(result.stderr)
    records.append({"name": name, "round": ROUND, "profile": str(profile), "profile_sha256": profile_hash,
                    "command": command, "status": result.returncode, "elapsed_seconds": time.monotonic() - started,
                    "source_files": [{"path": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in sources]})
    assert result.returncode == 0
aggregate = {}
for name in ("base", "ab"):
    totals = {key: 0 for key in ("arith_general_body", "whole_value")}
    for line in (OUT / f"{name}-inclusive-no.stdout").read_text().splitlines():
        match = re.match(r"\s*([0-9,]+)\s+", line)
        if match:
            for key in totals:
                if key in line:
                    totals[key] += int(match.group(1).replace(",", ""))
    aggregate[name] = totals
aggregate["difference"] = {key: aggregate["ab"][key] - aggregate["base"][key] for key in aggregate["base"]}
(OUT / "focused-aggregate.json").write_text(json.dumps(aggregate, indent=2) + "\n")
(OUT / "commands.json").write_text(json.dumps(records, indent=2) + "\n")
(OUT / "focused.tsv").write_text("\n".join(focused) + "\n")
print("\n".join(focused))
