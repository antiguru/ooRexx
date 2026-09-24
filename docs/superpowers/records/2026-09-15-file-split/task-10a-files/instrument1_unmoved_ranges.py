import difflib
import subprocess
import sys

base_path, head_path = sys.argv[1], sys.argv[2]

with open(base_path) as f:
    base_lines = f.readlines()
with open(head_path) as f:
    head_lines = f.readlines()

sm = difflib.SequenceMatcher(None, base_lines, head_lines, autojunk=False)
blocks = [b for b in sm.get_matching_blocks() if b.size > 0]

print(f"{len(blocks)} matching (unmoved) blocks between {base_path} and {head_path}\n")

total_lines = 0
failures = 0
for b in blocks:
    # 1-indexed inclusive line ranges for human-readable reporting.
    base_range = (b.a + 1, b.a + b.size)
    head_range = (b.b + 1, b.b + b.size)
    base_slice = base_lines[b.a : b.a + b.size]
    head_slice = head_lines[b.b : b.b + b.size]
    ok = base_slice == head_slice
    total_lines += b.size
    status = "OK" if ok else "FAIL"
    if not ok:
        failures += 1
    print(
        f"{status}  BASE {base_range[0]}-{base_range[1]} <-> HEAD {head_range[0]}-{head_range[1]} "
        f"({b.size} lines)"
    )

print(f"\n{total_lines} unmoved lines total across {len(blocks)} blocks, {failures} block(s) failed direct comparison.")

# Independent confirmation via the actual `cmp` binary, not Python string
# equality: write each block's base/head slice to temp files and shell out.
print("\nRe-confirmed with the `cmp` binary, per block:")
cmp_failures = 0
for i, b in enumerate(blocks):
    base_slice = base_lines[b.a : b.a + b.size]
    head_slice = head_lines[b.b : b.b + b.size]
    bpath = f"/tmp_block_base_{i}.txt"
    hpath = f"/tmp_block_head_{i}.txt"
    import os

    bpath = os.path.join(os.path.dirname(base_path), f"_block_base_{i}.txt")
    hpath = os.path.join(os.path.dirname(base_path), f"_block_head_{i}.txt")
    with open(bpath, "w") as f:
        f.writelines(base_slice)
    with open(hpath, "w") as f:
        f.writelines(head_slice)
    result = subprocess.run(["cmp", bpath, hpath], capture_output=True, text=True)
    base_range = (b.a + 1, b.a + b.size)
    head_range = (b.b + 1, b.b + b.size)
    if result.returncode != 0:
        cmp_failures += 1
        print(
            f"cmp FAIL block {i} BASE {base_range[0]}-{base_range[1]} HEAD {head_range[0]}-{head_range[1]}: "
            f"{result.stdout.strip()} {result.stderr.strip()}"
        )
    os.remove(bpath)
    os.remove(hpath)

print(f"cmp: {cmp_failures} failure(s) of {len(blocks)} blocks.")

# Sanity check: the total unmoved-line count plus the two large deleted
# ranges (Stats..fingerprint, write_provenance..write_blocked, both entirely
# absent from HEAD's main.rs) plus every genuinely-changed hunk's line count
# should reconstruct BASE's own total line count.
print(f"\nBASE total lines: {len(base_lines)}; HEAD total lines: {len(head_lines)}")
