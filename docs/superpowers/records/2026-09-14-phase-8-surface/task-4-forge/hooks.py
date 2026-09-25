#!/usr/bin/env python3
"""For each library exporting RexxGetPackage, read the package entry it answers
out of the binary (no dlopen) and report whether loader (+32) and unloader
(+40) are non-null, from the dynamic relocations at those offsets."""
import re, subprocess, sys

def run(*a):
    return subprocess.run(a, capture_output=True, text=True, check=True).stdout

def relocs(lib):
    out = {}
    for line in run('readelf', '-rW', lib).splitlines():
        m = re.match(r'\s*([0-9a-f]{8,16})\s+[0-9a-f]+\s+(R_X86_64_\w+)\s+(.*)$', line)
        if m:
            out[int(m.group(1), 16)] = (m.group(2), m.group(3).strip())
    return out

def symaddr(lib, name):
    for line in run('nm', '-D', '--defined-only', lib).splitlines():
        parts = line.split()
        if len(parts) == 3 and parts[2].split('@')[0] == name:
            return int(parts[0], 16)
    for line in run('nm', lib).splitlines():
        parts = line.split()
        if len(parts) == 3 and parts[2] == name:
            return int(parts[0], 16)
    return None

def target_of(reloc):
    kind, rest = reloc
    if kind == 'R_X86_64_RELATIVE':
        return int(rest, 16)
    return rest  # a symbol + addend

for lib in sys.argv[1:]:
    dis = run('objdump', '-d', '--no-show-raw-insn', lib)
    body = dis.split('<RexxGetPackage>:', 1)[1].split('\n\n', 1)[0]
    m = re.search(r'# ([0-9a-f]+) <', body)
    addr = int(m.group(1), 16)
    rel = relocs(lib)
    if 'lea' in body.split('#')[0].splitlines()[-1] or ' lea ' in body:
        entry = addr
    else:
        t = target_of(rel[addr])
        if isinstance(t, int):
            entry = t
        else:
            sym = t.split()[-1].split('+')[0].split('@')[0] if '+' not in t else t.split('+')[0].split()[-1].split('@')[0]
            entry = symaddr(lib, sym)
    hooks = []
    for off, name in ((32, 'loader'), (40, 'unloader')):
        r = rel.get(entry + off)
        hooks.append(f"{name}={'set ' + str(r) if r else 'null'}")
    print(f"{lib}: entry@{entry:#x} " + ' '.join(hooks))
