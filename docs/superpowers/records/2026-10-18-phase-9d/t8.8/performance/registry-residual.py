#!/usr/bin/env python3
"""Compare the registry pin with matched and historical source baselines."""

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
references = {"e1": ("c-e1-e2", "e1"), "e2": ("c-e1-e2", "e2"),
              "f_prior": ("e2-f", "f"), "f_matched": ("f-registry", "f"),
              "h": ("f-registry", "h")}
rows = []
ordinary = []
for program in profiles.PROGRAMS:
    values = {}
    for label, (group, name) in references.items():
        records = []
        for round_number in range(1, 4):
            path = Path(str(profiles.prefix(group, program, name, round_number)) + ".run.json")
            if not path.exists():
                break
            record = json.loads(path.read_text())
            if not record["valid"]:
                break
            records.append(record)
        if len(records) != 3:
            break
        values[label] = statistics.median(record["own"] for record in records)
    if len(values) != len(references):
        continue
    deltas = {name: 100 * (values["h"] - values[name]) / values[name] for name in references if name != "h"}
    row = {"program": program, **{name + "_own": value for name, value in values.items()},
           **{"h_vs_" + name + "_percent": delta for name, delta in deltas.items()}}
    rows.append(row)
    if program in ("dispatch", "dispatchclass", "sendloop"):
        ordinary.append(row)
directory = ROOT / "f-registry"
directory.mkdir(exist_ok=True)
fields = ["program", *[name + "_own" for name in references],
          *["h_vs_" + name + "_percent" for name in references if name != "h"]]
with (directory / "residual-vs-original.tsv").open("w") as output:
    writer = csv.DictWriter(output, fields, delimiter="\t", lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)
report = {
    "complete_programs": len(rows), "ordinary_send": ordinary,
    "reference_cohorts": references,
    "attribution_limits": "Independent cohorts compare different complete source revisions. H versus original e1 includes intervening e2 and f changes, rather than isolating the registry mechanism. A negative matched H versus F repair delta alone does not establish that the original regression was eliminated.",
}
(directory / "residual-analysis.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps(report, indent=2))
