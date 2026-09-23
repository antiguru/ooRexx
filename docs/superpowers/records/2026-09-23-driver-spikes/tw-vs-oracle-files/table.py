#!/usr/bin/env python3
"""Per-construct instructions per iteration from results.tsv.

usage: table.py RESULTS_TSV
Per construct: (program - control) / N, N = 200000; stem_read against ctrlfill.
Empty loop per iteration: (ctrl2 - ctrl) / 200000.
Columns: oracle ex-libc, oracle with libc, walker ex-libc, IR ex-libc, walker/oracle.
Asserts every rc is 0 and that the two rounds agree within 0.1 per iteration.
"""
import sys

N = 200000
rows = {}
for line in open(sys.argv[1]):
    name, rc, summary, libc, ld, ex, _ = line.rstrip("\n").split("\t")
    assert rc == "0", name
    rnd, prog, eng = name.split(".", 2)
    rows.setdefault((prog, eng), []).append((int(summary), int(ex)))


def val(prog, eng, which):
    vs = rows[(prog, eng)]
    assert len(vs) == 2, (prog, eng)
    i = 0 if which == "all" else 1
    return vs, sum(v[i] for v in vs) / 2


def per(prog, ctrl, eng, which):
    pv, p = val(prog, eng, which)
    cv, c = val(ctrl, eng, which)
    i = 0 if which == "all" else 1
    spread = max(abs((pv[0][i] - cv[0][i]) - (pv[1][i] - cv[1][i])) / N, 0)
    assert spread < 0.1, (prog, eng, which, spread)
    return (p - c) / N


order = ["ctrl2", "nop", "assign_var", "assign_lit", "incr", "concat", "if_eq",
         "stem_store", "stem_read", "bif_length", "call_r", "parse_var", "mul"]
print("construct\toracle_exlibc\toracle_all\ttw_exlibc\tir_exlibc\ttw/oracle\tir/oracle")
for p in order:
    ctrl = "ctrlfill" if p == "stem_read" else "ctrl"
    label = "empty loop (per iteration)" if p == "ctrl2" else p
    o = per(p, ctrl, "oracle", "ex")
    oa = per(p, ctrl, "oracle", "all")
    t = per(p, ctrl, "tree-walker", "ex")
    r = per(p, ctrl, "ir", "ex")
    print(f"{label}\t{o:.1f}\t{oa:.1f}\t{t:.1f}\t{r:.1f}\t{t / o:.2f}\t{r / o:.2f}")
