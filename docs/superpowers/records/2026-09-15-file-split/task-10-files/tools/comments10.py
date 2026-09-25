"""The per-commit comment pass: every comment that could have been made false
by this commit, listed for a reader to rule on line by line.

Lists, over POST's whole `rust/crates` tree (comments only, found with
rustlex) and `rust/corpus` (every line, since corpus files are prose):

  (a) every comment line naming a moved unit in backticks (`name`, `name(`,
      [`name`], `path::name`), the unit names taken from item-tool: keys of
      PRE's parent that POST has in a destination. In the parent and the
      destinations any such mention counts; anywhere else only one that
      reaches the name through the parent's module (`numeric::...::name`),
      since an unqualified mention elsewhere names that file's own item;
  (b) every comment line naming a file this commit touches, by its path
      tail (`builtin/numeric.rs`, `numeric.rs`, `numeric/tests.rs`, ...);
  (c) every comment line inside the moved text itself (the destinations)
      that uses a positional, relational or file-relative word ("above",
      "below", "before", "after", "ahead", "behind", "follow...",
      "preced...", "next to", "end of", "top of", "this file", "this
      module", "here", ...: POS_WORDS), wherever it names; and every
      such line left in the parent, since a positional word there can point
      at text that has moved away without naming any moved unit (Task 6's
      miss, found in Task 7's review).

The crate is SPLIT_CRATE (default `rexx-exec`), and the directory the
parent and destinations are relative to is SPLIT_ROOT (default `src`; Task
10's `gate_table_c.rs` is under `tests`).

usage: comments7.py PRE_SRC POST_RUST SPLIT_PARENT_RS DESTS(comma) [--tests]
"""
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import rustlex
import splitlib

pre_src, post_rust, parent, dests = sys.argv[1:5]
tests = "--tests" in sys.argv[5:]
dests = dests.split(",")
CRATE = os.environ.get("SPLIT_CRATE", "rexx-exec")
SRC = f"crates/{CRATE}/{os.environ.get('SPLIT_ROOT', 'src')}/"
post_src = os.path.join(post_rust, SRC)

pre_keys = {u["key"] for u in splitlib.load_units(os.path.join(pre_src, parent))}
moved = set()
for d in dests:
    prefix = os.path.basename(d)[: -len(".rs")] + "::" if tests else ""
    for u in splitlib.load_units(os.path.join(post_src, d)):
        k = prefix + u["key"]
        if k in pre_keys:
            moved.add(k)


def leaf(key):
    last = key.split("::")[-1]
    if last.startswith("macro ") and "/" in last:
        # `macro thread_local/NAME`: the static it declares.
        return last.split("/", 1)[1].split(",")[0]
    return last.split(" ")[-1].split("/")[0]


def is_use(key):
    return key.split("::")[-1].startswith("use ") or (" use " in key) or re.match(r"^(\w+::)*use ", key) is not None


names = sorted({leaf(k) for k in moved if not is_use(k) and not leaf(k).startswith("other@")})
alt = "|".join(map(re.escape, names))
name_re = re.compile(r"`(?:[\w:]*::)?(" + alt + r")(?:`|\()") if names else None
mod = os.path.basename(parent)[: -len(".rs")]
qual_re = re.compile(r"`(?:[\w:]*::)?" + re.escape(mod) + r"::(?:\w+::)*(" + alt + r")(?:`|\()") if names else None

tails = set()
for f in [parent] + dests:
    parts = f.split("/")
    for i in range(len(parts)):
        tails.add("/".join(parts[i:]))
    stem = f[: -len(".rs")]
    tails.add(stem + "/")
tails = sorted(t for t in tails if t not in ("tests.rs",))
file_re = re.compile(r"(?<![\w/])(" + "|".join(map(re.escape, tails)) + r")(?![\w])")
from poswords import POS_WORDS

pos_re = re.compile(r"\b(" + POS_WORDS + r")\b", re.I)


def comment_lines(path):
    text = open(path, errors="replace").read()
    starts = [0]
    for m in re.finditer("\n", text):
        starts.append(m.end())
    import bisect

    for kind, a, b in rustlex._scan(text):
        if kind not in ("comment", "doc", "block"):
            continue
        line0 = bisect.bisect_right(starts, a) - 1
        for k, l in enumerate(text[a:b].split("\n")):
            yield line0 + k + 1, l.strip()


hits_a, hits_b, hits_c = [], [], []
for root, _, files in os.walk(os.path.join(post_rust, "crates")):
    if "/target" in root:
        continue
    for f in sorted(files):
        if not f.endswith(".rs"):
            continue
        p = os.path.join(root, f)
        rel = os.path.relpath(p, post_rust)
        in_dest = any(rel == SRC + d for d in dests)
        in_parent = rel == SRC + parent
        local = in_dest or in_parent
        for n, l in comment_lines(p):
            if n <= 10 and l.startswith("/*"):
                continue
            if name_re and (name_re.search(l) if local else qual_re.search(l)):
                hits_a.append(f"{rel}:{n}: {l}")
            if file_re.search(l):
                hits_b.append(f"{rel}:{n}: {l}")
            if local and pos_re.search(l):
                hits_c.append(f"{rel}:{n}: {l}")
for root, _, files in os.walk(os.path.join(post_rust, "corpus")):
    for f in sorted(files):
        p = os.path.join(root, f)
        rel = os.path.relpath(p, post_rust)
        try:
            text = open(p, encoding="utf-8").read()
        except (UnicodeDecodeError, IsADirectoryError):
            continue
        for n, l in enumerate(text.split("\n"), 1):
            if qual_re and qual_re.search(l):
                hits_a.append(f"{rel}:{n}: {l.strip()}")
            if file_re.search(l):
                hits_b.append(f"{rel}:{n}: {l.strip()}")

print(f"moved units: {len(moved)}; names searched in backticks: {len(names)}")
print(f"file tails searched: {' '.join(tails)}")
for title, hits in (("(a) comments naming a moved unit", hits_a), ("(b) comments naming a touched file", hits_b), ("(c) positional words inside the destinations and the parent", hits_c)):
    print(f"\n{title}: {len(hits)}")
    for h in hits:
        print("  " + h)
