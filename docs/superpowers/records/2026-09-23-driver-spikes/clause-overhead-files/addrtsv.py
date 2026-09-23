#!/usr/bin/env python3
"""Per-instruction self Ir, program minus control, per iteration, as TSV with the
function, source position and disassembly; libc and ld-linux excluded.

usage: addrtsv.py BINARY PROGRAM_CG CONTROL_CG [N] > out.tsv
Columns: ir_per_it, addr, function, file, line, asm.
Needs --dump-instr=yes --compress-pos=no --compress-strings=no.
Only instructions in the object whose path ends with BINARY's basename are
disassembled; the others are kept with asm '?'.
"""
import os
import re
import subprocess
import sys


def load(path):
    out = {}
    ob = fn = fl = cur = None
    pending = False
    for line in open(path):
        if line.startswith("ob="):
            ob = line[3:].strip()
            continue
        if line.startswith("fl="):
            fl = cur = line[3:].strip()
            continue
        if line.startswith(("fi=", "fe=")):
            cur = line[3:].strip()
            continue
        if line.startswith("fn="):
            fn = line[3:].strip()
            cur = fl
            continue
        if line.startswith("calls="):
            pending = True
            continue
        if line.startswith("0x"):
            parts = line.split()
            if pending:
                pending = False
                continue
            if ob and ("libc.so.6" in ob or "ld-linux" in ob):
                continue
            k = (int(parts[0], 16), ob, fn, cur, int(parts[1]))
            out[k] = out.get(k, 0) + int(parts[-1])
    return out


binary, pcg, ccg = sys.argv[1:4]
N = int(sys.argv[4]) if len(sys.argv) > 4 else 200000
p = load(pcg)
c = load(ccg)
diff = {}
for k in set(p) | set(c):
    d = (p.get(k, 0) - c.get(k, 0)) / N
    if abs(d) >= 0.01:
        diff[k] = d
base = os.path.basename(binary)
mine = [k[0] for k in diff if k[1] and k[1].endswith(base)]
asm = {}
if mine:
    lo, hi = min(mine), max(mine) + 16
    dis = subprocess.run(
        ["objdump", "-d", "--no-show-raw-insn", "-C", f"--start-address={lo:#x}",
         f"--stop-address={hi:#x}", binary], capture_output=True, text=True).stdout
    for line in dis.splitlines():
        m = re.match(r"^\s+([0-9a-f]+):\s+(.*)$", line)
        if m:
            asm[int(m.group(1), 16)] = re.sub(r"\s+", " ", m.group(2).strip())
for k in sorted(diff, key=lambda k: (k[2] or "", k[0])):
    a, ob, fn, f, ln = k
    text = asm.get(a, "?") if ob and ob.endswith(base) else "?"
    short = "/".join((f or "?").split("/")[-2:])
    print(f"{diff[k]:.3f}\t{a:#x}\t{fn}\t{short}\t{ln}\t{text}")
