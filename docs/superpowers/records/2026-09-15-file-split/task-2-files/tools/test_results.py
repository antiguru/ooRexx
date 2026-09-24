"""Instrument 4: a `cargo test` log reduced to one line per test, keyed by
result block (the binary or doc-test block that ran it), plus each block's
own `test result:` counts.

usage: test_results.py LOG > RESULTS
       test_results.py --compare A B    exit 1 unless identical
"""
import re
import sys


def parse(path):
    block = None
    tests = []
    summaries = []
    for line in open(path, errors="replace"):
        line = line.rstrip("\n")
        m = re.match(r"\s*Running (\S+) \((.*)\)$", line)
        if m:
            binary = re.sub(r"-[0-9a-f]{16}$", "", m.group(2).split("/")[-1])
            block = f"{m.group(1)} [{binary}]"
            continue
        m = re.match(r"\s*Doc-tests (\S+)$", line)
        if m:
            block = f"doc-tests {m.group(1)}"
            continue
        m = re.match(r"test (.+?) \.\.\. (ok|FAILED|ignored.*)$", line)
        if m and block:
            tests.append(f"{block}\t{m.group(1)}\t{m.group(2)}")
            continue
        m = re.match(r"test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out", line)
        if m and block:
            summaries.append(f"{block}\tRESULT {m.group(1)} passed={m.group(2)} failed={m.group(3)} ignored={m.group(4)} measured={m.group(5)} filtered={m.group(6)}")
    return sorted(tests), sorted(summaries)


if sys.argv[1] == "--compare":
    a = parse(sys.argv[2])
    b = parse(sys.argv[3])
    if a == b:
        print(f"IDENTICAL: {len(a[0])} test lines, {len(a[1])} result blocks")
        sys.exit(0)
    import difflib

    for part, x, y in (("tests", a[0], b[0]), ("blocks", a[1], b[1])):
        for d in difflib.unified_diff(x, y, "before", "after", lineterm="", n=0):
            print(part, d)
    sys.exit(1)
else:
    tests, summaries = parse(sys.argv[1])
    for s in summaries:
        print(s)
    for t in tests:
        print(t)
