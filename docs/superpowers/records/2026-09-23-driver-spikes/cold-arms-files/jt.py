#!/usr/bin/env python3
"""Per-arm execution counts from the driver's jump tables.

usage: jt.py BIN SYMBOL_SUBSTR IR_RS CALLGRIND_OUT...

Finds every `lea TABLE(%rip); movslq (TABLE,IDX,4); add; jmp *` sequence in
the function whose mangled name contains SYMBOL_SUBSTR, reads one entry per
`Op` variant (the variants are read from IR_RS's `enum Op` in declaration
order, which is the tag order: the check below confirms it by addr2line),
and prints, per table and variant, the target address, the drive.rs line
addr2line gives the target, and the callgrind execution count of the
target's first instruction in each file. Indirect jumps whose table is
indexed by something other than the op tag are reported and skipped.
"""
import re
import struct
import subprocess
import sys

binary, symsub, ir_rs = sys.argv[1:4]
files = sys.argv[4:]

variants = []
in_enum = False
for line in open(ir_rs):
    if line.startswith("pub(crate) enum Op {"):
        in_enum = True
        continue
    if in_enum:
        if line.startswith("}"):
            break
        m = re.match(r"^    ([A-Z][A-Za-z]*)\b", line)
        if m:
            variants.append(m.group(1))

nm = subprocess.run(["nm", binary], capture_output=True, text=True).stdout
sym = [l.split()[2] for l in nm.splitlines() if symsub in l and l.split()[1] in "tT"]
if len(sym) != 1:
    sys.exit(f"symbol match: {sym}")
sym = sym[0]
start = int([l for l in nm.splitlines() if l.endswith(" " + sym)][0].split()[0], 16)
dis = subprocess.run(
    ["objdump", "-d", "--no-show-raw-insn", f"--disassemble={sym}", binary],
    capture_output=True, text=True,
).stdout.splitlines()

# section table for vaddr -> file offset
sh = subprocess.run(["readelf", "-SW", binary], capture_output=True, text=True).stdout
sections = []
for l in sh.splitlines():
    m = re.match(r"\s*\[\s*\d+\]\s+(\S+)\s+\S+\s+([0-9a-f]+)\s+([0-9a-f]+)\s+([0-9a-f]+)", l)
    if m:
        sections.append((m.group(1), int(m.group(2), 16), int(m.group(3), 16), int(m.group(4), 16)))
data = open(binary, "rb").read()


def read_i32(vaddr):
    for name, addr, off, size in sections:
        if addr and addr <= vaddr < addr + size:
            return struct.unpack_from("<i", data, off + vaddr - addr)[0]
    raise ValueError(hex(vaddr))


tables = []
for i, l in enumerate(dis):
    if re.search(r"jmp\s+\*%r", l):
        window = dis[max(0, i - 6):i]
        lea = [w for w in window if "lea" in w and "(%rip)" in w and "#" in w]
        tag = any(re.search(r"movzbl (0x0)?\(%r[a-z0-9]+\),", w) for w in window) or any("cmp    $0x28" in w for w in dis[max(0, i - 8):i])
        jaddr = l.split(":")[0].strip()
        if not lea:
            print(f"# indirect jump at {jaddr} has no rip-relative table, skipped")
            continue
        taddr = int(lea[-1].split("#")[1].split()[0], 16)
        tables.append((jaddr, taddr, tag))

counts = []
for path in files:
    c = {}
    fn = None
    skip = False
    for line in open(path):
        if line.startswith("fn="):
            fn = line[3:].strip()
            continue
        if line.startswith("calls="):
            skip = True
            continue
        m = re.match(r"^(0x[0-9a-f]+) (\d+) (\d+)", line)
        if not m:
            continue
        if skip:
            skip = False
            continue
        if fn and "run_ops_from" in fn and f"<{'true' if 'Kb1_' in sym else 'false'}>" in fn:
            c[int(m.group(1), 16)] = c.get(int(m.group(1), 16), 0) + int(m.group(3))
    lo = min(c) if c else start
    counts.append({a - (lo - start): v for a, v in c.items()})

names = [p.split("cg.")[-1] for p in files]
print("table\tjmp\tidx\tvariant\ttarget\tline\t" + "\t".join(names))
for jaddr, taddr, tag in tables:
    if not tag:
        print(f"# table at {taddr:#x} for jmp {jaddr} is not indexed by the op tag, skipped")
        continue
    targets = [taddr + read_i32(taddr + 4 * k) for k in range(len(variants))]
    lines = subprocess.run(
        ["addr2line", "-a", "-i", "-e", binary] + [hex(t) for t in targets],
        capture_output=True, text=True,
    ).stdout.split()
    chain = {}
    cur = None
    for tok in lines:
        if tok.startswith("0x"):
            cur = int(tok, 16)
            chain[cur] = []
        else:
            chain[cur].append(tok.rsplit("/", 1)[-1])
    for k, t in enumerate(targets):
        where = chain.get(t, ["?"])[-1]
        cs = "\t".join(str(c.get(t, 0)) for c in counts)
        print(f"{taddr:#x}\t{jaddr}\t{k}\t{variants[k]}\t{t:#x}\t{where}\t{cs}")
