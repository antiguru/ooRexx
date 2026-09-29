#!/usr/bin/env python3
"""Insert COUNT never-called functions, kept by a #[used] static, into TREE's
rust/crates/rexx-exec/src/ir/drive.rs.

usage: layout-pad.py TREE COUNT
TREE is an extracted copy of the repository, not the working tree.
"""
import sys
from pathlib import Path

tree, count = Path(sys.argv[1]), int(sys.argv[2])
drive = tree / "rust/crates/rexx-exec/src/ir/drive.rs"
text = drive.read_text()
anchor = "\nimpl Interp {\n"
assert text.count(anchor) >= 1 and "layout_pad_" not in text, drive
fns = []
for k in range(count):
    body = "\n".join(
        f"    v = (v ^ {0x9E3779B97F4A7C15 + k * 31 + j:#x}).wrapping_mul({2 * (k * 7 + j) + 1}).rotate_left({(k + j) % 63 + 1});"
        for j in range(8)
    )
    fns.append(f"#[inline(never)]\nfn layout_pad_{k}(x: u64) -> u64 {{\n    let mut v = x;\n{body}\n    v\n}}\n")
names = ", ".join(f"layout_pad_{k}" for k in range(count))
pad = "\n" + "\n".join(fns) + f"\n#[used]\nstatic LAYOUT_PAD: [fn(u64) -> u64; {count}] = [{names}];\n"
at = text.index(anchor)
drive.write_text(text[:at] + "\n" + pad + text[at:])
print(f"{drive}: {count} pad functions")
