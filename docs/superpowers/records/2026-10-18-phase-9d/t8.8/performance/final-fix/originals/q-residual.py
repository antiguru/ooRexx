#!/usr/bin/env python3
"""Report Q residual instruction costs against retained historical cohorts."""

import csv
import importlib.util
import json
from pathlib import Path
import statistics

ROOT = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("profiles", ROOT / "profile-slices.py")
profiles = importlib.util.module_from_spec(spec)
spec.loader.exec_module(profiles)
profiles.verify_inputs()
references = {"base": ("base-ab-c", "base"), "e1": ("c-e1-e2", "e1"),
              "e2": ("e2-f", "e2"), "f": ("e2-f", "f"),
              "p": ("p-q", "p"), "q": ("p-q", "q")}
values = {}
for program in profiles.PROGRAMS:
    values[program] = {}
    for label, (group, name) in references.items():
        records = [json.loads(Path(str(profiles.prefix(group, program, name, r)) + ".run.json").read_text())
                   for r in range(1, 4)]
        assert all(record["valid"] and record["interpreter_status"] == "0" for record in records)
        values[program][label] = statistics.median(record["own"] for record in records)

report = {
    "complete_programs": len(values), "reference_cohorts": references,
    "reference_pins": {name: {"path": str(profiles.PINS[name]), "sha256": profiles.EXPECTED[name]}
                       for name in references},
    "attribution_limits": "Q versus original e1, original e2, corrected F, and pre-8.8 BASE compares separate cohorts and complete source revisions with intervening semantic and performance changes. These residuals do not isolate method-entry lookup, PARSE, numeric handling, or another source mechanism. Each residual above +0.05% is explicitly OVER, independently of a negative direct Q/P repair delta.",
    "residuals": {},
}
directory = ROOT / "p-q"
for baseline in ("base", "e1", "e2", "f"):
    rows = []
    for program, medians in values.items():
        delta = 100 * (medians["q"] - medians[baseline]) / medians[baseline]
        rows.append({"program": program, "reference_cohort": references[baseline][0],
                     "q_cohort": "p-q", "reference_own_median": medians[baseline],
                     "q_own_median": medians["q"], "q_vs_reference_percent": delta,
                     "residual_gate": "OVER" if delta > 0.05 else "PASS"})
    with (directory / f"q-vs-{baseline}-residual.tsv").open("w") as output:
        writer = csv.DictWriter(output, list(rows[0]), delimiter="\t", lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)
    report["residuals"][baseline] = {
        "over_budget": [row for row in rows if row["residual_gate"] == "OVER"],
        "ordinary_send": [row for row in rows if row["program"] in ("dispatch", "dispatchclass", "sendloop")],
        "worst": max(rows, key=lambda row: row["q_vs_reference_percent"]),
    }
(directory / "residual-analysis.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps(report, indent=2))
