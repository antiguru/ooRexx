"""Sum each axis's recorded per-task change over the valid tasks (void and
unmeasured cells excluded). usage: sum_recorded.py per-task-recorded.tsv"""
import sys
rows = [l.rstrip("\n").split("\t") for l in open(sys.argv[1]) if not l.startswith("#")]
head, body = rows[0], rows[1:]
for i, axis in enumerate(head[2:], 2):
    vals = [float(r[i]) for r in body if r[1] == "valid" and r[i] != "-"]
    tasks = [r[0] for r in body if r[1] == "valid" and r[i] != "-"]
    print(f"{axis}\t{sum(vals):+.5f}%\tfrom tasks {','.join(tasks)}")
