import re
import sys
import os

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


def find_start(lines, kind, name, from_idx=0):
    pat = re.compile(
        r"^(pub\(super\)\s+|pub\(crate\)\s+|pub\s+)?" + re.escape(kind) + r"\s+" + re.escape(name) + r"\b"
    )
    for i in range(from_idx, len(lines)):
        if pat.match(lines[i]):
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


base_path, head_path, out_dir = sys.argv[1], sys.argv[2], sys.argv[3]
base = read(base_path)
head = read(head_path)

os.makedirs(os.path.join(out_dir, "base"), exist_ok=True)
os.makedirs(os.path.join(out_dir, "head"), exist_ok=True)

for name, kind in names:
    bidx = find_start(base, kind, name)
    hidx = find_start(head, kind, name)
    blines = extract_item(base, bidx, is_const=(kind == "const"))
    hlines = extract_item(head, hidx, is_const=(kind == "const"))
    with open(os.path.join(out_dir, "base", f"{name}.rs"), "w") as f:
        f.write("\n".join(blines) + "\n")
    with open(os.path.join(out_dir, "head", f"{name}.rs"), "w") as f:
        f.write("\n".join(hlines) + "\n")

# Stats struct + impl: written as one syn::File-parseable unit (two items,
# concatenated with a blank line, exactly as they appear in each source).
bidx = find_start(base, "struct", "Stats")
blines = extract_item(base, bidx, is_const=False)
after = bidx + len(blines)
while base[after].strip() == "":
    after += 1
bidx2 = find_start(base, "impl", "Stats", from_idx=after)
blines2 = extract_item(base, bidx2, is_const=False)

hidx = find_start(head, "struct", "Stats")
hlines = extract_item(head, hidx, is_const=False)
hafter = hidx + len(hlines)
while head[hafter].strip() == "":
    hafter += 1
hidx2 = find_start(head, "impl", "Stats", from_idx=hafter)
hlines2 = extract_item(head, hidx2, is_const=False)

with open(os.path.join(out_dir, "base", "Stats.rs"), "w") as f:
    f.write("\n".join(blines) + "\n\n" + "\n".join(blines2) + "\n")
with open(os.path.join(out_dir, "head", "Stats.rs"), "w") as f:
    f.write("\n".join(hlines) + "\n\n" + "\n".join(hlines2) + "\n")

print("wrote", len(names) + 1, "item pairs to", out_dir)
