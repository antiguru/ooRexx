#!/usr/bin/env python3
"""Classify executed stack traffic per instruction, using LLVM's stack-frame-layout
remarks to tell register-allocator spill slots from genuine stack objects.

usage: spills.py BINARY REMARKS CG [CONTROL_CG N] [--slots FUNCSUBSTR] [--rows FUNCSUBSTR]

Every executed instruction in BINARY with an `off(%rsp)` operand is placed in the
frame by the function's prologue (pushes P, `sub $F,%rsp`): remark offset =
off - F - 8P. The remark says what LLVM put there: `Spill` (a register-allocator
slot), `Variable` (an alloca: an array, a value built in place, an address-taken
local) or `Fixed` (incoming stack arguments). Categories:
  spill-store   mov reg -> spill slot
  spill-reload  mov spill slot -> reg
  spill-fold    a spill slot read as an ALU/cmp/test operand (no instruction of its own)
  spill-rmw     ALU op writing a spill slot in place
  obj           any access to a Variable slot (incl. lea of it)
  arg           read of a Fixed slot (a stack-passed argument)
  push/pop      callee-saved register save/restore
  frame         sub/add of %rsp
  unknown-rsp   rsp operand the prologue model cannot place (rsp moved mid-function)
  other         everything else
Counts are self Ir, (CG - CONTROL)/N when a control is given, CG/N otherwise.
"""
import re
import subprocess
import sys
from collections import defaultdict

args = [a for a in sys.argv[1:] if not a.startswith("--")]
opts = {}
av = sys.argv[1:]
for i, a in enumerate(av):
    if a.startswith("--"):
        opts[a[2:]] = av[i + 1]
        args.remove(av[i + 1]) if av[i + 1] in args else None
binary, remarks, cg = args[0], args[1], args[2]
if len(args) == 4:
    ctl, N = None, float(args[3])
else:
    ctl = args[3] if len(args) > 3 else None
    N = float(args[4]) if len(args) > 4 else 1.0

# --- remarks: mangled name -> {offset: (type, size, names)}
layouts = {}
cur = None
last = None
for line in open(remarks, errors="replace"):
    s = line.strip()
    if s.startswith("Function: "):
        cur = s[len("Function: "):]
        layouts.setdefault(cur, {})
        last = None
        continue
    m = re.match(r"Offset: \[SP([+-]\d+)\], Type: (\w+), Align: \d+, Size: (\d+)", s)
    if m and cur is not None:
        off = int(m.group(1))
        layouts[cur][off] = [m.group(2), int(m.group(3)), []]
        last = off
        continue
    m = re.match(r"^(\S+) @ (\S+):(\d+)$", s)
    if m and cur is not None and last is not None:
        f = m.group(2).split("/")[-1]
        layouts[cur][last][2].append(f"{m.group(1)}@{f}:{m.group(3)}")
        continue
    if s.startswith("note:") or s == "":
        if s.startswith("note:"):
            cur = None
        continue

# --- symbols
syms = []
for line in subprocess.run(["nm", "-S", "--defined-only", binary], capture_output=True, text=True).stdout.splitlines():
    p = line.split()
    if len(p) == 4 and p[2] in "tTwW":
        syms.append((int(p[0], 16), int(p[1], 16), p[3]))
syms.sort()
starts = [s[0] for s in syms]
import bisect


def sym_of(a):
    i = bisect.bisect_right(starts, a) - 1
    if i >= 0 and syms[i][0] <= a < syms[i][0] + max(syms[i][1], 1):
        return syms[i]
    return None


# --- callgrind per address
def load(path):
    out = defaultdict(float)
    ob = None
    pending = False
    for line in open(path):
        if line.startswith("ob="):
            ob = line[3:].strip()
            continue
        if line.startswith("calls="):
            pending = True
            continue
        if line.startswith("0x"):
            if pending:
                pending = False
                continue
            if ob and ("libc.so.6" in ob or "ld-linux" in ob):
                continue
            if ob and not ob.endswith(binary.split("/")[-1]):
                continue
            p = line.split()
            out[int(p[0], 16)] += int(p[-1])
    return out


cnt = load(cg)
if ctl:
    c2 = load(ctl)
    for k, v in c2.items():
        cnt[k] -= v
cnt = {k: v / N for k, v in cnt.items() if abs(v / N) >= (0.005 if ctl else 0)}

# --- disassemble the functions that executed
funcs = {}
for a in cnt:
    s = sym_of(a)
    if s:
        funcs[s[2]] = s
dis = {}
prolog = {}
for name, (lo, size, _) in funcs.items():
    out = subprocess.run(["objdump", "-d", "--no-show-raw-insn", f"--start-address={lo:#x}",
                          f"--stop-address={lo + size:#x}", binary], capture_output=True, text=True).stdout
    ins = []
    for line in out.splitlines():
        m = re.match(r"^\s+([0-9a-f]+):\s+(.*)$", line)
        if m:
            txt = re.sub(r"\s+", " ", m.group(2).strip())
            dis[int(m.group(1), 16)] = txt
            ins.append(txt)
    pushes = 0
    frame = 0
    for t in ins[:12]:
        if t.startswith("push "):
            pushes += 1
        m = re.match(r"sub \$0x([0-9a-f]+),%rsp", t)
        if m:
            frame = int(m.group(1), 16)
            break
    prolog[name] = (pushes, frame)

RSP = re.compile(r"(-?0x[0-9a-f]+|-?\d+)?\(%rsp(?:,[^)]*)?\)")


def classify(name, a):
    t = dis.get(a)
    if t is None:
        return "other", None
    op = t.split(" ")[0]
    if op == "push":
        return "push/pop", None
    if op == "pop":
        return "push/pop", None
    if re.match(r"(sub|add) \$0x[0-9a-f]+,%rsp$", t):
        return "frame", None
    m = RSP.search(t)
    if not m:
        return "other", None
    off = int(m.group(1), 16) if m.group(1) and "x" in m.group(1) else int(m.group(1) or 0)
    pushes, frame = prolog.get(name, (0, 0))
    roff = off - frame - 8 * pushes
    lay = layouts.get(name, {})
    slot = None
    for o, (ty, sz, names) in lay.items():
        if o <= roff < o + sz:
            slot = (o, ty, sz, names)
            break
    if slot is None:
        return "unknown-rsp", (roff, "?", 0, [])
    ty = slot[1]
    if ty == "Variable":
        return "obj", slot
    if ty == "Fixed":
        return "arg", slot
    # Spill
    operands = t[len(op) + 1:]
    memlast = operands.rstrip().endswith(")") and "(%rsp" in operands.split(",")[-1] or re.search(r"\(%rsp[^)]*\)$", operands)
    if op.startswith(("mov", "vmov")):
        return ("spill-store" if memlast else "spill-reload"), slot
    if op.startswith(("cmp", "test", "bt")):
        return "spill-fold", slot
    if memlast and "," in operands:
        return "spill-rmw", slot
    if memlast:  # single-operand like incq/decq/notq
        return "spill-rmw", slot
    return "spill-fold", slot


cat = defaultdict(float)
byfn = defaultdict(lambda: defaultdict(float))
byslot = defaultdict(float)
rows = []
for a, v in cnt.items():
    s = sym_of(a)
    name = s[2] if s else "?"
    c, slot = classify(name, a) if s else ("other", None)
    cat[c] += v
    byfn[name][c] += v
    if slot and c.startswith("spill"):
        byslot[(name, slot[0], ",".join(slot[3][:4]))] += v
    rows.append((a, v, name, c, slot))


def dem(n):
    return subprocess.run(["rustfilt"], input=n, capture_output=True, text=True).stdout.strip() if False else n


total = sum(cat.values())
print(f"total\t{total:.3f}")
for c in ["spill-store", "spill-reload", "spill-fold", "spill-rmw", "obj", "arg", "push/pop", "frame", "unknown-rsp", "other"]:
    print(f"{c}\t{cat.get(c, 0):.3f}")
print(f"spill-total\t{cat['spill-store'] + cat['spill-reload'] + cat['spill-fold'] + cat['spill-rmw']:.3f}")
print(f"spill-extra\t{cat['spill-store'] + cat['spill-reload'] + cat['spill-rmw']:.3f}")
print("\n# per function (spill store/reload/rmw, obj, push/pop, total)")
fl = sorted(byfn.items(), key=lambda kv: -sum(kv[1].values()))
dm = {}
names = [n for n, _ in fl[:40]]
if names:
    out = subprocess.run(["c++filt"], input="\n".join(names), capture_output=True, text=True).stdout.splitlines()
    dm = dict(zip(names, out))
for n, d in fl[:40]:
    print(f"{d['spill-store']:.2f}\t{d['spill-reload'] + d['spill-fold']:.2f}\t{d['spill-rmw']:.2f}\t{d['obj']:.2f}\t{d['push/pop']:.2f}\t{sum(d.values()):.2f}\t{dm.get(n, n)[:90]}")
print("\n# spill slots by executed traffic")
sl = sorted(byslot.items(), key=lambda kv: -kv[1])
acc = 0
st = sum(v for v in byslot.values())
for (n, o, nm), v in sl[:60]:
    acc += v
    print(f"{v:.2f}\t{100 * acc / st if st else 0:.1f}%\t{dm.get(n, n)[-30:]}\tSP{o:+d}\t{nm}")
if "rows" in opts:
    print("\n# rows")
    for a, v, name, c, slot in sorted(rows, key=lambda r: r[0]):
        if opts["rows"] in (dm.get(name, name)) and c != "other":
            print(f"{v:.2f}\t{a:#x}\t{c}\t{('SP%+d' % slot[0]) if slot else ''}\t{dis.get(a, '?')}")
