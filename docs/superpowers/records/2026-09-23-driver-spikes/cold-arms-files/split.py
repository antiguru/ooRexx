#!/usr/bin/env python3
"""Split each variant's round-1 delta against base into: the driver's own
self Ir (every run_ops_from and run_activation symbol, since `full` inlines
the driver into run_activation), the hot callees whose inlining moved, and
everything else. Deltas in instructions and per cent of base ex-libc.

usage: split.py M_DIR
"""
import importlib.util
import sys

spec = importlib.util.spec_from_file_location("byfn", __file__.replace("split.py", "byfn_lib.py"))
M = sys.argv[1]


def load(path):
    fn = None
    skip = False
    out = {}
    for line in open(path):
        if line.startswith("fn="):
            fn = line[3:].strip()
            continue
        if line.startswith("calls="):
            skip = True
            continue
        if line[:1].isdigit() or line[:1] in "+-*":
            parts = line.split()
            if len(parts) < 2:
                continue
            if skip:
                skip = False
                continue
            out[fn] = out.get(fn, 0) + int(parts[-1])
    return out


def exlibc(path):
    for line in open(f"{M}/results.tsv"):
        f = line.split("\t")
        if f[0] == path:
            return int(f[4])


print("| variant | axis | driver self | callees whose inlining moved | other | total |")
print("|---|---|---:|---:|---:|---:|")
for v in ["full", "fullni", "half"]:
    for a in ["rexxcps", "varlookup", "arith", "emptyloop", "dispatch", "compound"]:
        b = load(f"{M}/cg.1.{a}.base")
        h = load(f"{M}/cg.1.{a}.{v}")
        base_ex = exlibc(f"1.{a}.base")
        drv = inl = oth = 0
        for k in set(b) | set(h):
            d = h.get(k, 0) - b.get(k, 0)
            if "run_ops_from" in k or "run_activation" in k:
                drv += d
            elif "arith_small_int" in k or "LoopHeaderValues" in k or "arith_general" in k:
                inl += d
            elif "libc.so" in k or k.startswith(("_int_", "__mem", "malloc", "free")):
                continue
            else:
                oth += d
        pct = lambda x: f"{x:+,} ({x / base_ex * 100:+.3f}%)"
        print(f"| {v} | `{a}` | {pct(drv)} | {pct(inl)} | {pct(oth)} | {pct(drv + inl + oth)} |")
