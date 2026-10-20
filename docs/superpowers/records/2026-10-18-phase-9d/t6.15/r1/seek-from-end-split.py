# Counts the generated rows labelled `49 seek-from-end` by which position the
# character SEEK <n at the first differing item names. Run from the repository
# root: python3 -I docs/superpowers/records/2026-10-18-phase-9d/t6.15/r1/seek-from-end-split.py
import collections
R = 'docs/superpowers/records/2026-10-18-phase-9d/t6.15/r1'
T = 'rust/crates/rexx-exec/tests/stream_model/departures-oracle.tsv'
labels = {}
for line in open(T):
    if line.startswith('#') or not line.strip():
        continue
    name, label = line.rstrip('\n').split('\t')
    labels[name] = label
def items(answer):
    ops, content = answer.rsplit(' [', 1)
    return [x for x in ops.split(';') if x] + ['[' + content]
counts = collections.Counter()
for line in open(R + '/dump/oracle.tsv'):
    name, _, _, ops, native, model = line.rstrip('\n').split('\t')
    if labels.get(name) != '49 seek-from-end':
        continue
    ops = ops.split(' ; ')
    at = next(k for k, (a, b) in enumerate(zip(items(native), items(model))) if a != b)
    op = ops[at]
    which = 'write' if ' write' in op else 'read' if ' read' in op else 'neither'
    counts[which] += 1
for which, count in sorted(counts.items()):
    print(count, which)
