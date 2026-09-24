import re
import sys

# Every top-level item that moved from the original file into report.rs,
# named by its bare identifier (function/struct/const name). The same name
# is searched for in both files; a leading pub(super)/pub(crate)/pub and any
# #[allow(...)] attribute line immediately above are tolerated and stripped
# before comparing token streams.
names = [
    ("TARGET_COVERAGE", "const"),
    ("interval_for", "fn"),
    ("interval_coverage", "fn"),
    ("joint_coverage_bound", "fn"),
    ("ratio_interval_caveat", "fn"),
    ("seconds", "fn"),
    ("fingerprint", "fn"),
    ("write_provenance", "fn"),
    ("write_offset", "fn"),
    ("counter_median", "fn"),
    ("write_counters", "fn"),
    ("write_axes", "fn"),
    ("write_rexxcps", "fn"),
    ("self_timed_figures", "fn"),
    ("median_of", "fn"),
    ("write_self_timed", "fn"),
    ("quoted_line", "fn"),
    ("parse_cps", "fn"),
    ("write_blocked", "fn"),
]


def read(path):
    with open(path) as f:
        return f.read().split("\n")


def find_start(lines, kind, name):
    # Match a line at column 0 that is (optionally) "pub(super) "/"pub(crate) "/"pub "
    # followed by exactly "<kind> <name>" as its own token (bounded by '(' , ' ', or '{').
    pat = re.compile(
        r"^(pub\(super\)\s+|pub\(crate\)\s+|pub\s+)?" + re.escape(kind) + r"\s+" + re.escape(name) + r"\b"
    )
    for i, line in enumerate(lines):
        if pat.match(line):
            return i
    raise SystemExit(f"not found: {kind} {name}")


def extract_item(lines, start_idx, is_const):
    if is_const:
        return [lines[start_idx]]
    depth = 0
    collected = []
    started_brace = False
    for j in range(start_idx, len(lines)):
        line = lines[j]
        collected.append(line)
        depth += line.count("{") - line.count("}")
        if "{" in line:
            started_brace = True
        if started_brace and depth == 0:
            break
    return collected


def normalize(lines):
    text = "\n".join(lines)
    text = re.sub(r"\bpub\(super\)\s+", "", text)
    text = re.sub(r"\bpub\(crate\)\s+", "", text)
    return re.sub(r"\s+", "", text)


base = read(sys.argv[1])
report = read(sys.argv[2])

mismatches = 0
for name, kind in names:
    bidx = find_start(base, kind, name)
    ridx = find_start(report, kind, name)
    blines = extract_item(base, bidx, is_const=(kind == "const"))
    rlines = extract_item(report, ridx, is_const=(kind == "const"))
    bn = normalize(blines)
    rn = normalize(rlines)
    ok = bn == rn
    print(f"{'OK  ' if ok else 'FAIL'} {name}: base_len={len(bn)} report_len={len(rn)}")
    if not ok:
        mismatches += 1
        for a, b in zip(bn, rn):
            if a != b:
                break
        print("  BASE:  ", bn[:300])
        print("  REPORT:", rn[:300])

# Stats struct + impl, handled separately (two consecutive top-level items).
bidx = find_start(base, "struct", "Stats")
blines = extract_item(base, bidx, is_const=False)
after = bidx + len(blines)
while base[after].strip() == "":
    after += 1
bidx2 = find_start(base[after:], "impl", "Stats")
blines2 = extract_item(base[after:], bidx2, is_const=False)
base_all = blines + blines2

ridx = find_start(report, "struct", "Stats")
rlines = extract_item(report, ridx, is_const=False)
rafter = ridx + len(rlines)
while report[rafter].strip() == "":
    rafter += 1
ridx2 = find_start(report[rafter:], "impl", "Stats")
rlines2 = extract_item(report[rafter:], ridx2, is_const=False)
report_all = rlines + rlines2

bn = normalize(base_all)
rn = normalize(report_all)
ok = bn == rn
print(f"{'OK  ' if ok else 'FAIL'} Stats(struct+impl): base_len={len(bn)} report_len={len(rn)}")
if not ok:
    mismatches += 1

print()
print("TOTAL MISMATCHES:", mismatches, "of", len(names) + 1)
