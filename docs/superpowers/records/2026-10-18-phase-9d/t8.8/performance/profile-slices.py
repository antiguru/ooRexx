#!/usr/bin/env python3
"""Checkpoint the existing all-program callgrind instrument in bounded batches."""

import concurrent.futures
import datetime
import difflib
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import signal
import statistics
import subprocess
import sys
import time

REPO = Path("/home/moritz/dev/repos/ooRexx-rust-rewrite")
RECORDS = Path(__file__).resolve().parent
RAW = Path("/tmp/codex-p9d-t88-performance")
HERE = REPO / "rust/bench-programs"
LIB = Path("/home/moritz/dev/repos/ooRexx/build/lib")
PROGRAMS = "alloc alloc4c arith assign churn compound decloop decrender define dirread dispatch dispatchclass emptyloop extcall fibcall fibfunc hashnew heapshape intcall interpret nop parse pkglocal sayloop sendloop startup strings textnum varlookup rexxcps pingguard pingmsg pingsem".split()
GROUPS = {"base-ab-c": ["base", "ab", "c"], "c-e1-e2": ["c", "e1", "e2"], "e2-f": ["e2", "f"], "f-registry": ["f", "h"], "h-a-fperf": ["h", "a", "p"]}
PINS = {name: Path("/tmp/codex-p9d-t88-pins") / filename for name, filename in {
    "base": "8.8-base-rexx", "ab": "8.8ab-rexx", "c": "8.8c-rexx",
    "e1": "8.8e1-rexx", "e2": "8.8e2-rexx", "f": "8.8f-r1-rexx",
    "h": "8.8e2-perf-r1-rexx", "a": "8.8a-perf-r1-rexx",
    "p": "8.8f-perf-r1-rexx"}.items()}
EXPECTED = {
    "base": "4913ad067898fd6a786c12a696b2a21b0d066ad6b019c73273f78e30c16f5c58",
    "ab": "361392c36b1207a9174dfab7b32b2acb212d2c69e53f3e76f106e3a1cdad42dd",
    "c": "af52ecd6802b878e9d81be6440492f6bfdadd680628cb48b3224615c70d39a78",
    "e1": "434d6567458248fdb804eebe9edb715a54ddb9ded650fcca994eed6c4b691e06",
    "e2": "00e29fde4456ee4845a76941263f495b827c083af35363cb54eb3ac8c6b7c733",
    "f": "b695b78fe24903b4be5a2874c8cd128193bbe165f92b50a4d6a8a9956d03d3be",
    "h": "470dcb41db61ab7a4d64f3276ce28e539d5686d130be245a4ec7b30e6d6725ac",
    "a": "ecb06e984ed736233ce0e2ef779b74941488ae64fdec10333dfb2215c309198a",
    "p": "6ed1c08eb03d54c48d1fe3947ce376f09502aaeba34ac221c334b01b1572b86e",
}


def sha(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def program_path(program):
    if program == "rexxcps":
        return REPO / "rust/bench-rexxcps/rexxcps.rex"
    return HERE / ("pingpong" if program.startswith("ping") else "") / f"{program}.rex"


def prepare():
    RAW.mkdir(exist_ok=False)
    script = (HERE / "callgrind.sh").read_text()
    modified = script.replace('here=$(cd "$(dirname "$0")" && pwd)', f"here={HERE}")
    modified = modified.replace("LD_LIBRARY_PATH=$lib valgrind --tool=callgrind", "LD_LIBRARY_PATH=$lib memcap 2G timeout -k 5s 480s valgrind --tool=callgrind")
    assert modified != script and modified.count("memcap 2G timeout") == 1
    (RAW / "callgrind-capped.sh").write_text(modified)
    one = modified[:modified.index("running=0\n")] + '\none "$PROFILE_BINARY_INDEX" "$PROFILE_PROGRAM" "$PROFILE_ROUND"\nfi\n'
    (RAW / "callgrind-one.sh").write_text(one)
    (RECORDS / "callgrind-capped.diff").write_text("".join(difflib.unified_diff(script.splitlines(True), modified.splitlines(True), fromfile="original/callgrind.sh", tofile="owned/callgrind-capped.sh")))
    (RECORDS / "callgrind-checkpoint-extraction.diff").write_text("".join(difflib.unified_diff(modified.splitlines(True), one.splitlines(True), fromfile="owned/callgrind-capped.sh", tofile="owned/callgrind-one.sh")))
    inputs = [HERE / "callgrind.sh", HERE / "cgsum.py", *map(program_path, PROGRAMS)]
    inputs += [LIB / "liborxfunction.so", RAW / "callgrind-capped.sh", RAW / "callgrind-one.sh"]
    manifest = {
        "prepared_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "source_revision": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip(),
        "programs": PROGRAMS, "rounds": 3, "groups": GROUPS,
        "files": [{"path": str(path), "sha256": sha(path), "bytes": path.stat().st_size} for path in inputs],
        "pins": [{"name": name, "path": str(path), "sha256": sha(path), "expected_sha256": EXPECTED[name]} for name, path in PINS.items()],
        "valgrind_version": subprocess.check_output(["valgrind", "--version"], text=True).strip(),
        "memory_caps": {"driver": "8G", "profile": "2G", "jobs": 3},
        "profile_timeout_seconds": 480, "batch_deadline_seconds": 525,
        "tmp_free_bytes": shutil.disk_usage("/tmp").free,
    }
    assert all(pin["sha256"] == pin["expected_sha256"] for pin in manifest["pins"])
    write_json(RECORDS / "inputs.json", manifest)
    shutil.copyfile(RAW / "callgrind-capped.sh", RECORDS / "callgrind-capped.sh.txt")
    shutil.copyfile(RAW / "callgrind-one.sh", RECORDS / "callgrind-one.sh.txt")
    print(json.dumps(manifest, indent=2))


def verify_inputs():
    manifest = json.loads((RECORDS / "inputs.json").read_text())
    for item in manifest["files"] + manifest["pins"]:
        assert sha(item["path"]) == item["sha256"], item["path"]
    for filename in ("additional-f-inputs.json", "registry-inputs.json", "numeric-inputs.json", "f-performance-inputs.json"):
        extra = RECORDS / filename
        if extra.exists():
            for item in json.loads(extra.read_text())["pins"]:
                assert Path(item["path"]) == PINS[item["name"]]
                assert item["sha256"] == EXPECTED[item["name"]]
                assert sha(item["path"]) == item["sha256"], item["path"]


def prefix(group, program, name, round_number):
    return RAW / group / f"{program}.{name}.r{round_number}"


def valid(group, program, name, round_number):
    path = Path(str(prefix(group, program, name, round_number)) + ".run.json")
    return path.exists() and json.loads(path.read_text())["valid"]


def memory_sample(pid):
    pending = [pid]
    samples = []
    while pending:
        owned = pending.pop()
        proc = Path("/proc") / str(owned)
        try:
            fields = dict(line.split(":", 1) for line in (proc / "status").read_text().splitlines() if ":" in line)
            pending.extend(int(child) for child in (proc / "task" / str(owned) / "children").read_text().split())
            samples.append({"pid": owned, "command": (proc / "cmdline").read_bytes().replace(b"\0", b" ").decode(),
                            "rss_kib": int(fields.get("VmRSS", "0 kB").split()[0]),
                            "hwm_kib": int(fields.get("VmHWM", "0 kB").split()[0])})
        except (FileNotFoundError, ProcessLookupError):
            pass
    return samples


def run_one(group, program, round_number, index, hard_stop, jobs):
    name = GROUPS[group][index]
    path = prefix(group, program, name, round_number)
    for existing in path.parent.glob(path.name + ".*"):
        destination = Path(str(existing) + f".invalid-{time.time_ns()}")
        existing.rename(destination)
    command = ["bash", str(RAW / "callgrind-one.sh"), "-r", "3", "-j", str(jobs), "-o", str(RAW / group), "-B", "0.05"]
    command += [f"{item}={PINS[item]}" for item in GROUPS[group]]
    env = dict(os.environ, PROFILE_BINARY_INDEX=str(index), PROFILE_PROGRAM=program,
               PROFILE_ROUND=str(round_number), REXX_LIB_DIR=str(LIB))
    started = time.monotonic()
    with Path(str(path) + ".driver.stdout").open("wb") as out, Path(str(path) + ".driver.stderr").open("wb") as err:
        process = subprocess.Popen(command, cwd=REPO / "rust", env=env,
                                   stdout=out, stderr=err, start_new_session=True)
        expired = False
        memory = {"subprocess_pid": process.pid, "max_tree_rss_kib": 0, "max_child_hwm_kib": 0, "peak_sample": []}
        while True:
            sample = memory_sample(process.pid)
            rss = sum(item["rss_kib"] for item in sample)
            hwm = max((item["hwm_kib"] for item in sample), default=0)
            if rss > memory["max_tree_rss_kib"]:
                memory.update(max_tree_rss_kib=rss, peak_sample=sample)
            memory["max_child_hwm_kib"] = max(memory["max_child_hwm_kib"], hwm)
            write_json(Path(str(path) + ".live.json"), memory)
            try:
                status = process.wait(timeout=min(1, max(0.01, hard_stop - time.monotonic())))
                break
            except subprocess.TimeoutExpired:
                if time.monotonic() >= hard_stop:
                    expired = True
                    owned_groups = set()
                    for item in memory_sample(process.pid):
                        try:
                            if os.getsid(item["pid"]) == process.pid:
                                owned_groups.add(os.getpgid(item["pid"]))
                        except ProcessLookupError:
                            pass
                    for group_id in owned_groups:
                        try:
                            os.killpg(group_id, signal.SIGKILL)
                        except ProcessLookupError:
                            pass
                    status = process.wait()
                    break
    record = {"group": group, "program": program, "name": name, "round": round_number,
              "binary_index": index, "command": command, "cwd": str(REPO / "rust"),
              "profile_environment": {key: env[key] for key in ("PROFILE_BINARY_INDEX", "PROFILE_PROGRAM", "PROFILE_ROUND", "REXX_LIB_DIR")},
              "status": status, "batch_expired": expired, "elapsed_seconds": time.monotonic() - started,
              "finished_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "memory": memory, "valid": False}
    rc_path = Path(str(path) + ".rc")
    row_path = Path(str(path) + ".row")
    if rc_path.exists():
        record["interpreter_status"] = rc_path.read_text().strip()
    if row_path.exists():
        row = row_path.read_text().strip().split("\t")
        record["row"] = row
        record["valid"] = status == 0 and len(row) == 8 and row[-1] == "rc=0" and row[3:7] != [""] * 4
    if record["valid"]:
        record["whole"], record["libc"], record["ld"], record["own"] = map(int, row[3:7])
    for suffix in ("out", "err"):
        item = Path(str(path) + "." + suffix)
        if item.exists():
            record[suffix + "_sha256"] = sha(item)
            record[suffix + "_bytes"] = item.stat().st_size
    write_json(Path(str(path) + ".run.json"), record)
    return record


def run_batch(group, label, maximum, jobs):
    assert 1 <= jobs <= 8
    if group == "e2-f":
        authorization = RECORDS / "additional-f-inputs.json"
        assert authorization.exists() and json.loads(authorization.read_text())["profiling_authorized"], "Corrected f pin is not authorized for profiling yet"
    if group == "f-registry":
        authorization = RECORDS / "registry-inputs.json"
        assert authorization.exists() and json.loads(authorization.read_text())["profiling_authorized"], "Registry repair pin is not authorized for profiling yet"
    if group == "h-a-fperf":
        for filename in ("numeric-inputs.json", "f-performance-inputs.json"):
            authorization = RECORDS / filename
            assert authorization.exists() and json.loads(authorization.read_text())["profiling_authorized"], "Accepted repair pins are required for this cohort"
    verify_inputs()
    driver_hash = sha(__file__)
    snapshots = RECORDS / "orchestrators"
    snapshots.mkdir(exist_ok=True)
    snapshot = snapshots / f"{driver_hash}.py"
    if not snapshot.exists():
        shutil.copyfile(__file__, snapshot)
    batch_path = RECORDS / f"{label}.json"
    assert not batch_path.exists(), batch_path
    (RAW / group).mkdir(exist_ok=True)
    started = time.monotonic()
    hard_stop = started + 525
    history = [json.loads(path.read_text()) for path in RAW.glob("*/*.run.json")]
    records = [item for item in history if item["valid"]]
    counts = {}
    historical = REPO / "docs/superpowers/records/2026-10-18-phase-9d/t8.7a/r1/callgrind.txt"
    for line in historical.read_text().splitlines()[1:]:
        fields = line.split("\t")
        counts[fields[0]] = int(fields[1])
    priority = ["startup", "dispatch", "dispatchclass", "intcall", "sendloop"]
    if group in ("base-ab-c", "h-a-fperf"):
        priority.insert(1, "arith")
    tasks = [(p, r) for p in priority for r in range(1, 4)]
    tasks += [(p, r) for r in range(1, 4) for p in PROGRAMS if p not in priority]
    active = {}
    completed = []
    scheduled = 0
    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as pool:
        while True:
            for program, round_number in tasks:
                if len(active) >= jobs or scheduled >= maximum or any(not item["valid"] for item in completed):
                    break
                if (program, round_number) in active.values():
                    continue
                indices = [(k + round_number - 1) % len(GROUPS[group]) for k in range(len(GROUPS[group]))]
                missing = [i for i in indices if not valid(group, program, GROUPS[group][i], round_number)]
                if not missing:
                    continue
                measured = [r["elapsed_seconds"] for r in records if r["program"] == program]
                rates = [r["whole"] / r["elapsed_seconds"] for r in records]
                estimate = min(480, 2 * max(measured) + 10) if measured else (min(480, counts[program] / min(rates) * 2 + 10) if rates else 480)
                if hard_stop - time.monotonic() < estimate + 10:
                    continue
                index = missing[0]
                future = pool.submit(run_one, group, program, round_number, index, hard_stop - 3, jobs)
                active[future] = (program, round_number)
                scheduled += 1
            if not active:
                break
            done, _ = concurrent.futures.wait(active, timeout=10, return_when=concurrent.futures.FIRST_COMPLETED)
            for future in done:
                active.pop(future)
                item = future.result()
                records.append(item) if item["valid"] else None
                completed.append(item)
                print(json.dumps({key: item.get(key) for key in ("group", "program", "name", "round", "valid", "elapsed_seconds", "interpreter_status")}), flush=True)
    verify_inputs()
    total = sum(valid(group, p, name, r) for p in PROGRAMS for name in GROUPS[group] for r in range(1, 4))
    result = {"group": group, "jobs": jobs, "driver_sha256": driver_hash, "driver_snapshot": str(snapshot), "driver_command": ["memcap", "8G", "python3", str(Path(__file__).resolve()), "run", group, label, str(maximum), str(jobs)],
              "deadline_seconds": 525, "elapsed_seconds": time.monotonic() - started,
              "completed_profiles": completed, "valid_profiles_total": total, "expected_profiles": len(PROGRAMS) * 3 * len(GROUPS[group]),
              "tmp_free_bytes": shutil.disk_usage("/tmp").free,
              "memory_available": next(line for line in Path("/proc/meminfo").read_text().splitlines() if line.startswith("MemAvailable:")),
              "cpu_count": os.cpu_count()}
    result["driver_status"] = 0 if all(item["valid"] for item in completed) else 1
    write_json(batch_path, result)
    print(json.dumps({key: value for key, value in result.items() if key != "completed_profiles"}), flush=True)
    return result["driver_status"]


if __name__ == "__main__":
    if sys.argv[1] == "prepare":
        prepare()
    elif sys.argv[1] == "run":
        raise SystemExit(run_batch(sys.argv[2], sys.argv[3], int(sys.argv[4]), int(sys.argv[5]) if len(sys.argv) > 5 else 3))
    else:
        raise SystemExit("Expected prepare or run")
