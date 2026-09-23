#!/usr/bin/env python3
"""Classifies the walker's per-clause instructions for `nop` (addrdiff output over
every function, nop minus ctrl) into the report's classes by source line and
address range. Rules are first-match; every instruction lands in exactly one class.

usage: classify_nop.py AD_NOP_ALL_TXT
"""
import re
import sys

HOIST = (0x13728F, 0x137469)  # the entry block of walk_step_in_temps_frame: loads + spills

RULES = [
    ("codegen: hoisted loads/spills at entry", lambda a, f, l: HOIST[0] <= a <= HOIST[1]),
    ("codegen: prologue/epilogue", lambda a, f, l: (f, l) in {
        ("tree_walker.rs", 112), ("tree_walker.rs", 148), ("run.rs", 985), ("run.rs", 1598)}),
    ("Rust checks (bounds, Option)", lambda a, f, l: f in {"index.rs", "option.rs"}),
    ("error propagation (Result/ClauseOutcome)", lambda a, f, l: f == "result.rs"
        or (f, l) in {("run.rs", 5028), ("run.rs", 5037), ("run.rs", 5041), ("run.rs", 5049),
                      ("run.rs", 5052), ("tree_walker.rs", 130), ("tree_walker.rs", 146)}
        or (f == "run.rs" and l == 0 and 0x13AB00 <= a <= 0x13AD00)
        or (f == "run.rs" and l == 0 and 0x15D6DF <= a <= 0x15D6EB)),
    ("bookkeeping the oracle also does", lambda a, f, l: (f, l) in {
        ("run.rs", 4906), ("run.rs", 4907), ("run.rs", 4919), ("activation.rs", 201),
        ("clause.rs", 273), ("clause.rs", 274), ("clause.rs", 317), ("clause.rs", 322),
        ("clause.rs", 265), ("run.rs", 4980), ("mod.rs", 3064), ("mod.rs", 3045),
        ("mod.rs", 1721), ("mod.rs", 1831), ("mod.rs", 1836), ("unix.rs", 0)}
        or (f == "mod.rs" and l == 0 and 0x1377EE <= a <= 0x137810)),
    ("eager trace/debug state (oracle: lazy or in execute)", lambda a, f, l: f in {"trace.rs", "plan.rs"}
        or (f, l) in {("run.rs", 5459), ("run.rs", 5460), ("run.rs", 5463), ("run.rs", 4945),
                      ("run.rs", 4949), ("activation.rs", 1096), ("run.rs", 8325), ("run.rs", 8336),
                      ("run.rs", 8337), ("run.rs", 5081), ("tree_walker.rs", 138), ("mod.rs", 967),
                      ("mod.rs", 968)}
        or (f == "run.rs" and l == 0 and 0x1374C0 <= a <= 0x1376B0)
        or (f in {"tree_walker.rs", "mod.rs"} and l == 0 and 0x13AC92 <= a <= 0x13ACC4)),
    ("call layering / dispatch", lambda a, f, l: (f == "tree_walker.rs" and l in {97, 99, 101, 119, 120, 176, 262, 0})
        or (f, l) in {("run.rs", 993), ("run.rs", 0), ("mod.rs", 1873)}),
]

tot = {}
unclassified = []
for line in open(sys.argv[1]):
    m = re.match(r"^\s*(-?[\d.]+) (0x[0-9a-f]+) (\S+):(\d+)", line)
    if not m:
        continue
    d, a, f, l = float(m.group(1)), int(m.group(2), 16), m.group(3), int(m.group(4))
    for name, rule in RULES:
        if rule(a, f, l):
            tot[name] = tot.get(name, 0) + d
            break
    else:
        unclassified.append(line.rstrip())
for k, v in tot.items():
    print(f"{v:7.1f}  {k}")
print(f"{sum(tot.values()):7.1f}  total classified")
for u in unclassified:
    print("UNCLASSIFIED", u)
