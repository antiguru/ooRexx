#!/usr/bin/env python3
"""Report completed own-BASE instruction comparisons from checkpointed profiles."""

import csv
import importlib.util
import json
from pathlib import Path
import statistics
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("profiles", ROOT / "profile-slices.py")
profiles = importlib.util.module_from_spec(spec)
spec.loader.exec_module(profiles)
profiles.verify_inputs()
TIMED = {"heapshape", "rexxcps"}
relations = {"base-ab-c": [("8.8a", "base", "ab"), ("8.8c", "ab", "c")],
             "c-e1-e2": [("8.8e1", "c", "e1"), ("8.8e2", "e1", "e2")],
             "e2-f": [("8.8f", "e2", "f")],
             "f-registry": [("8.8h", "f", "h")],
             "h-a-fperf": [("8.8a-repair", "h", "a"), ("8.8f-repair", "a", "p")],
             "p-q": [("8.8f-repair-r2", "p", "q")]}
partial = "--partial" in sys.argv
report = {"groups": {}, "over_budget": [], "provisional_over_budget": [], "output_errors": [], "invalid_profiles": []}

for group, names in profiles.GROUPS.items():
    directory = ROOT / group
    directory.mkdir(exist_ok=True)
    records = {}
    for path in (profiles.RAW / group).glob("*.run.json"):
        try:
            item = json.loads(path.read_text())
        except json.JSONDecodeError:
            continue
        if item["valid"]:
            records[(item["program"], item["name"], item["round"])] = item
        else:
            report["invalid_profiles"].append(item)
    complete = [program for program in profiles.PROGRAMS
                if all((program, name, r) in records for name in names for r in range(1, 4))]
    if not partial:
        assert len(complete) == len(profiles.PROGRAMS), (group, len(complete))
    output_rows = []
    for (program, name, round_number), item in records.items():
        prefix = profiles.prefix(group, program, name, round_number)
        for suffix in ("out", "err"):
            assert profiles.sha(str(prefix) + "." + suffix) == item[suffix + "_sha256"]
        baseline = records.get((program, names[0], 1))
        agrees = "TIMED" if program in TIMED else ("yes" if baseline and item["out_sha256"] == baseline["out_sha256"] else "missing baseline" if not baseline else "NO")
        output_rows.append([program, name, round_number, item["interpreter_status"], agrees,
                            item["out_bytes"], item["out_sha256"], item["err_bytes"], item["err_sha256"], item["elapsed_seconds"]])
        if agrees == "NO" or item["interpreter_status"] != "0" or item["err_bytes"]:
            report["output_errors"].append({"group": group, "program": program, "name": name, "round": round_number,
                                            "stdout_agreement": agrees, "status": item["interpreter_status"], "stderr_bytes": item["err_bytes"]})
    with (directory / "output-agreement.tsv").open("w") as output:
        writer = csv.writer(output, delimiter="\t", lineterminator="\n")
        writer.writerow(["program", "binary", "round", "status", "stdout_agreement", "stdout_bytes", "stdout_sha256", "stderr_bytes", "stderr_sha256", "seconds"])
        writer.writerows(output_rows)
    rows = ["\t".join(records[(p, name, r)]["row"]) for p in complete for r in range(1, 4) for name in names]
    (directory / "summary.tsv").write_text("\n".join(rows) + ("\n" if rows else ""))
    summary = {"valid_profiles": len(records), "expected_profiles": len(profiles.PROGRAMS) * 3 * len(names),
               "complete_programs": len(complete), "spreads": []}
    with (directory / "matched-round-deltas.tsv").open("w") as output:
        writer = csv.writer(output, delimiter="\t", lineterminator="\n")
        writer.writerow(["slice", "program", "round", "base", "head", "base_own", "head_own", "own_delta_percent", "median_set_complete"])
        for label, base, head in relations[group]:
            for program in profiles.PROGRAMS:
                for round_number in range(1, 4):
                    if (program, base, round_number) not in records or (program, head, round_number) not in records:
                        continue
                    base_own = records[(program, base, round_number)]["own"]
                    head_own = records[(program, head, round_number)]["own"]
                    delta = 100 * (head_own - base_own) / base_own
                    writer.writerow([label, program, round_number, base, head, base_own, head_own, f"{delta:+.8f}", program in complete])
                    if program not in complete and delta > 0.05:
                        report["provisional_over_budget"].append({"slice": label, "program": program, "round": round_number, "own_delta_percent": delta})
    for label, base, head in relations[group]:
        with (directory / f"{label}-own-delta.tsv").open("w") as output:
            writer = csv.writer(output, delimiter="\t", lineterminator="\n")
            writer.writerow(["program", "base", "head", "base_own", "head_own", "own_delta_percent", "whole_delta_percent",
                             "base_whole_spread_percent", "head_whole_spread_percent", "base_own_spread_percent", "head_own_spread_percent", "own_gate"])
            for program in complete:
                values = {name: {kind: [records[(program, name, r)][kind] for r in range(1, 4)] for kind in ("whole", "own")} for name in (base, head)}
                mid = {name: {kind: statistics.median(values[name][kind]) for kind in ("whole", "own")} for name in (base, head)}
                delta = {kind: 100 * (mid[head][kind] - mid[base][kind]) / mid[base][kind] for kind in ("whole", "own")}
                spread = {name: {kind: 100 * (max(values[name][kind]) - min(values[name][kind])) / mid[name][kind] for kind in ("whole", "own")} for name in (base, head)}
                over = delta["own"] > 0.05
                writer.writerow([program, base, head, mid[base]["own"], mid[head]["own"], f'{delta["own"]:+.8f}', f'{delta["whole"]:+.8f}',
                                 f'{spread[base]["whole"]:.8f}', f'{spread[head]["whole"]:.8f}', f'{spread[base]["own"]:.8f}', f'{spread[head]["own"]:.8f}', "OVER" if over else "PASS"])
                if over:
                    report["over_budget"].append({"slice": label, "program": program, "own_delta_percent": delta["own"], "base": base, "head": head})
                for name in (base, head):
                    if spread[name]["whole"] > 0.01:
                        entry = {"program": program, "name": name, "whole_spread_percent": spread[name]["whole"], "own_spread_percent": spread[name]["own"]}
                        if entry not in summary["spreads"]:
                            summary["spreads"].append(entry)
    if len(complete) == len(profiles.PROGRAMS):
        (profiles.RAW / group / "summary.tsv").write_text((directory / "summary.tsv").read_text())
        command = ["bash", str(profiles.RAW / "callgrind-capped.sh"), "-T", "-o", str(profiles.RAW / group), "-B", "0.05"]
        command += [f"{name}={profiles.PINS[name]}" for name in names]
        result = subprocess.run(command, cwd=profiles.REPO / "rust", capture_output=True, timeout=30)
        (directory / "original-table.stdout").write_bytes(result.stdout)
        (directory / "original-table.stderr").write_bytes(result.stderr)
        summary["original_table_command"] = command
        summary["original_table_status"] = result.returncode
    report["groups"][group] = summary

profiles.verify_inputs()
(ROOT / ("partial-analysis.json" if partial else "analysis.json")).write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps(report, indent=2))
