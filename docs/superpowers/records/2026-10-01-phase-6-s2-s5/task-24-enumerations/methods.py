# python3 docs/superpowers/records/2026-10-01-phase-6-s2-s5/task-24-enumerations/methods.py ORACLE_TREE
# Every method of the oracle's Message, EventSemaphore and MutexSemaphore
# tables (interpreter/memory/Setup.cpp) joined to rexx-exec's dispatch.rs rows,
# run from the repository root. Prints MISSING for a method no row answers.
import re, sys
setup = open(sys.argv[1] + '/interpreter/memory/Setup.cpp').read().split('\n')
src = open('rust/crates/rexx-exec/src/dispatch.rs').read()
rows, table = [], None
for m in re.finditer(r'static\s+([A-Z_]+)\s*:|\(\s*"(Message|EventSemaphore|MutexSemaphore)",\s*"([^"]+)"', src):
    if m.group(1):
        table = m.group(1)
        continue
    rows.append((src.count('\n', 0, m.start()) + 1, table, m.group(2), m.group(3)))
want = []
for lo, hi, cls in [(1050, 1082, 'Message'), (1325, 1341, 'EventSemaphore'), (1348, 1362, 'MutexSemaphore')]:
    for i in range(lo - 1, hi):
        m = re.search(r'Add(Class|Unguarded|)Method\("([A-Za-z]+)"', setup[i])
        if m:
            want.append((cls, m.group(2).upper(), m.group(1) == 'Class', i + 1))
for cls, name, is_class, line in want:
    hits = [f'dispatch.rs:{r[0]} {r[1]}' for r in rows
            if r[2] == cls and r[3] == name and ('CLASS' in r[1]) == is_class]
    print(f"Setup.cpp:{line}\t{cls}\t{'class ' if is_class else ''}{name}\t{', '.join(hits) or 'MISSING'}")
