#!/usr/bin/env python3
"""Per-instruction self Ir, program minus control, per iteration, annotated with
disassembly and source line, for functions matching a substring.

usage: addrdiff.py BINARY PROGRAM_CG CONTROL_CG FUNC_SUBSTRING [N] [ADDR_BIAS_HEX]
Needs --dump-instr=yes --compress-pos=no --compress-strings=no.
ADDR_BIAS is subtracted from callgrind addresses before looking them up in the
binary (0 for this PIE executable; the load base for a shared library).
Prints each executed-difference instruction, then a classification summary:
stack traffic (an operand through %rsp/%rbp), calls, conditional branches,
other; and a per-source-line total.
"""
import re
import subprocess
import sys


def load(path, sub):
    out = {}
    fn = None
    fl = cur = None
    pending = False
    for line in open(path):
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
            if fn and sub in fn:
                a = int(parts[0], 16)
                k = (a, fn, cur, int(parts[1]))
                out[k] = out.get(k, 0) + int(parts[-1])
    return out


binary, pcg, ccg, sub = sys.argv[1:5]
N = int(sys.argv[5]) if len(sys.argv) > 5 else 200000
bias = int(sys.argv[6], 16) if len(sys.argv) > 6 else 0
p = load(pcg, sub)
c = load(ccg, sub)
diff = {}
for k in set(p) | set(c):
    d = (p.get(k, 0) - c.get(k, 0)) / N
    if abs(d) >= 0.5:
        diff[k] = d
if not diff:
    print("no difference")
    sys.exit()
lo = min(k[0] for k in diff) - bias
hi = max(k[0] for k in diff) - bias + 16
dis = subprocess.run(
    ["objdump", "-d", "--no-show-raw-insn", "-C", f"--start-address={lo:#x}",
     f"--stop-address={hi:#x}", binary],
    capture_output=True, text=True).stdout
asm = {}
for line in dis.splitlines():
    m = re.match(r"^\s+([0-9a-f]+):\s+(.*)$", line)
    if m:
        asm[int(m.group(1), 16)] = m.group(2).strip()
cls = {"stack": 0.0, "call": 0.0, "jcc": 0.0, "other": 0.0}
byline = {}
for k in sorted(diff):
    a, fn, f, ln = k
    d = diff[k]
    text = asm.get(a - bias, "?")
    short = (f or "?").rsplit("/", 1)[-1]
    if re.search(r"\((%rsp|%rbp)", text) and not text.startswith(("call", "lea")):
        c_ = "stack"
    elif text.startswith("call"):
        c_ = "call"
    elif re.match(r"j(?!mp)", text):
        c_ = "jcc"
    else:
        c_ = "other"
    cls[c_] += d
    byline[(short, ln)] = byline.get((short, ln), 0) + d
    print(f"{d:6.2f} {a - bias:#x} {short}:{ln:<5} {c_:5} {text[:90]}")
print("-- classes", {k: round(v, 2) for k, v in cls.items()}, "total", round(sum(cls.values()), 2))
print("-- by line")
for (f, ln), d in sorted(byline.items(), key=lambda x: -x[1]):
    print(f"{d:6.2f} {f}:{ln}")
