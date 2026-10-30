#!/usr/bin/env python3
"""Verify complete cohorts against their retained profiles and input hashes."""

import csv
import datetime
import importlib.util
import itertools
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("profiles", ROOT / "profile-slices.py")
profiles = importlib.util.module_from_spec(spec)
spec.loader.exec_module(profiles)
label, *groups = sys.argv[1:]
assert groups
profiles.verify_inputs()
disk = [path.stem for path in profiles.HERE.glob("*.rex")]
disk += [path.stem for path in (profiles.HERE / "pingpong").glob("*.rex")]
disk += ["rexxcps"]
assert sorted(disk) == sorted(profiles.PROGRAMS)
verified = []
diagnostics = []
for group in groups:
    names = profiles.GROUPS[group]
    for program in profiles.PROGRAMS:
        for round_number in range(1, 4):
            for name in names:
                prefix = profiles.prefix(group, program, name, round_number)
                item = json.loads(Path(str(prefix) + ".run.json").read_text())
                assert item["valid"] and item["status"] == 0 and not item["batch_expired"]
                assert item["interpreter_status"] == "0"
                assert item["own"] + item["libc"] + item["ld"] == item["whole"]
                assert Path(str(prefix) + ".rc").read_text().strip() == "0"
                assert Path(str(prefix) + ".row").read_text().strip().split("\t") == item["row"]
                assert item["command"][-len(names):] == [f"{candidate}={profiles.PINS[candidate]}" for candidate in names]
                with Path(str(prefix) + ".cg").open() as stream:
                    header = list(itertools.islice(stream, 40))
                command = next(line for line in header if line.startswith("cmd:"))
                arguments = command.split(":", 1)[1].split()
                assert len(arguments) == 2, command
                assert Path(arguments[0]).resolve() == profiles.PINS[name].resolve(), command
                assert Path(arguments[1]).resolve() == profiles.program_path(program).resolve(), command
                total = int(next(line for line in header if line.startswith("summary:")).split()[1])
                assert total == item["whole"]
                for suffix in ("out", "err"):
                    assert profiles.sha(str(prefix) + "." + suffix) == item[suffix + "_sha256"]
                driver_errors = Path(str(prefix) + ".driver.stderr")
                if driver_errors.stat().st_size:
                    diagnostics.append({"path": str(driver_errors), "bytes": driver_errors.stat().st_size})
                verified.append([group, program, name, round_number, item["whole"], item["libc"], item["ld"], item["own"], profiles.sha(str(prefix) + ".cg")])
    assert len(list((profiles.RAW / group).glob("*.run.json"))) == len(profiles.PROGRAMS) * len(names) * 3
profiles.verify_inputs()
manifest = ROOT / f"{label}-profile-manifest.tsv"
with manifest.open("w") as output:
    writer = csv.writer(output, delimiter="\t", lineterminator="\n")
    writer.writerow(["group", "program", "binary", "round", "whole", "libc", "ld", "own", "raw_profile_sha256"])
    writer.writerows(verified)
result = {"verified_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "groups": groups,
          "programs_per_cohort": len(profiles.PROGRAMS), "rounds": 3, "verified_profiles": len(verified),
          "raw_manifest": str(manifest), "raw_manifest_sha256": profiles.sha(manifest),
          "driver_diagnostics": diagnostics,
          "checks": ["program enumeration", "pin and instrument hashes", "successful status", "row identity", "libc/loader arithmetic", "actual profiled argv", "raw summary", "independent output hashes"]}
(ROOT / f"{label}-verification.json").write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps(result, indent=2))
