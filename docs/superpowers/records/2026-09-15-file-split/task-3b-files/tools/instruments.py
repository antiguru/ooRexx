"""Instruments 1-3 for one commit of a file split (SPLIT_PARENT, default
`dispatch`: the parent is `<SPLIT_PARENT>.rs`, the children `<SPLIT_PARENT>/*.rs`).

usage: instruments.py PRE_ROOT POST_ROOT REMOVED_JSON OUT_PREFIX [--tests]

PRE_ROOT and POST_ROOT are `crates/rexx-exec/src` directories (the parent
commit's and this commit's, e.g. two `git worktree`s or `git archive`
extractions). REMOVED_JSON is the list of 0-based line indices of PRE's
`dispatch.rs` the move removed, as `splitlib.take` recorded them.

Instrument 1 (`OUT_PREFIX-instrument1.txt`):
  (a) the unmoved lines: PRE's `dispatch.rs` with the removed indices taken
      out must equal POST's `dispatch.rs` except for inserted lines and
      visibility-only edits, each listed; every equal block is also written
      to a file pair and compared with `cmp`;
  (b) the moved lines: each moved unit's PRE text (its span, comments above
      it included) is compared with `cmp` against the same unit's POST text,
      after undoing only a visibility change on the declaration line (listed)
      and, with --tests, the four-space de-indent.
Instrument 2 (`OUT_PREFIX-instrument2.txt`): per unit key present on both
  sides, the token stream with the visibility removed; plus the keys present
  on one side only, and every visibility change.
Instrument 3 (`OUT_PREFIX-instrument3.txt`): per unit key, the decoded
  literal list, and the item tool's decodable count cross-checked against the
  independent lexer's count of the same text, on both sides.

Exit status 1 if any check fails.
"""
import difflib
import re
import json
import os
import subprocess
import sys
import tempfile

import rustlex
import splitlib

PARENT = os.environ.get("SPLIT_PARENT", "dispatch")
PARENT_RS = PARENT + ".rs"
from splitlib import drop_trailing_commas, squash

pre_root, post_root, removed_json, out_prefix = sys.argv[1:5]
tests_mode = "--tests" in sys.argv[5:]
# A unit this commit edits on purpose (the chain in `ObjectModel::build`):
# its token difference is listed rather than failed; instrument 1 (a) shows
# the edited lines themselves.
expected_edits = [a.split("=", 1)[1] for a in sys.argv[5:] if a.startswith("--expect-edit=")]

CHILDREN = sorted(
    os.path.join(PARENT, f)
    for f in os.listdir(os.path.join(post_root, PARENT))
    if f.endswith(".rs")
)
pre_children = set(
    os.path.join(PARENT, f)
    for f in os.listdir(os.path.join(pre_root, PARENT))
    if f.endswith(".rs")
)
NEW_FILES = [c for c in CHILDREN if c not in pre_children]
failures = []


def read_lines(path):
    lines = open(path).read().split("\n")
    assert lines[-1] == ""
    return lines[:-1]


def cmp_bytes(a, b, tag):
    with tempfile.TemporaryDirectory() as d:
        pa, pb = os.path.join(d, "a"), os.path.join(d, "b")
        open(pa, "w").write(a)
        open(pb, "w").write(b)
        r = subprocess.run(["cmp", pa, pb], capture_output=True, text=True)
        return r.returncode == 0, (r.stdout + r.stderr).strip()


VIS = ("pub(crate) ", "pub(super) ", "pub ")


def strip_vis_line(line):
    s = line.lstrip(" ")
    ind = line[: len(line) - len(s)]
    for v in VIS:
        if s.startswith(v):
            return ind + s[len(v) :], v.strip()
    return line, None


# ---------------------------------------------------------------- units
pre_units = {u["key"]: u for u in splitlib.load_units(os.path.join(pre_root, PARENT_RS))}
pre_units_file = {k: PARENT_RS for k in pre_units}
post_units = {}
post_units_file = {}
for rel in [PARENT_RS] + NEW_FILES:
    for u in splitlib.load_units(os.path.join(post_root, rel)):
        key = u["key"]
        if tests_mode and rel == os.path.join(PARENT, "tests.rs"):
            key = "tests::" + key
        assert key not in post_units, ("duplicate key across files", key, rel)
        post_units[key] = u
        post_units_file[key] = rel

pre_dispatch = read_lines(os.path.join(pre_root, PARENT_RS))
post_lines_cache = {}


def post_lines(rel):
    if rel not in post_lines_cache:
        post_lines_cache[rel] = read_lines(os.path.join(post_root, rel))
    return post_lines_cache[rel]


# ---------------------------------------------------------------- instrument 1
def declared_line(post_index):
    """Whether a POST dispatch.rs line (0-based) is inside a declared edit."""
    for key in expected_edits:
        if post_units_file.get(key) == PARENT_RS:
            a, b = splitlib.span(read_lines(os.path.join(post_root, PARENT_RS)), post_units[key])
            if a <= post_index <= b:
                return True
    return False


out1 = []
removed = set(json.load(open(removed_json)))
expected = [l for i, l in enumerate(pre_dispatch) if i not in removed]
actual = read_lines(os.path.join(post_root, PARENT_RS))
sm = difflib.SequenceMatcher(a=expected, b=actual, autojunk=False)
equal_blocks = 0
equal_lines = 0
out1.append(f"(a) unmoved lines: PRE {PARENT_RS} minus the removed lines, against POST {PARENT_RS}")
for tag, i1, i2, j1, j2 in sm.get_opcodes():
    if tag == "equal":
        ok, msg = cmp_bytes("\n".join(expected[i1:i2]) + "\n", "\n".join(actual[j1:j2]) + "\n", "eq")
        equal_blocks += 1
        equal_lines += i2 - i1
        if not ok:
            failures.append(f"I1a cmp failed on an equal block {i1}-{i2}: {msg}")
    elif tag == "insert":
        for j in range(j1, j2):
            out1.append(f"  INSERTED post:{j + 1}: {actual[j]}")
    elif tag == "replace" and (i2 - i1) == (j2 - j1):
        for k in range(i2 - i1):
            before, after = expected[i1 + k], actual[j1 + k]
            stripped, vis = strip_vis_line(after)
            before_stripped, before_vis = strip_vis_line(before)
            if stripped == before_stripped and vis != before_vis:
                out1.append(f"  VISIBILITY post:{j1 + k + 1}: {before_vis or 'private'} -> {vis or 'private'}: {after.strip()}")
            elif declared_line(j1 + k):
                out1.append(f"  CHANGED (declared edit) post:{j1 + k + 1}: {before!r} -> {after!r}")
            else:
                failures.append(f"I1a changed unmoved line pre-expected:{i1 + k + 1} post:{j1 + k + 1}: {before!r} -> {after!r}")
                out1.append(f"  CHANGED {before!r} -> {after!r}")
    else:
        failures.append(f"I1a {tag} expected[{i1}:{i2}] actual[{j1}:{j2}]")
        out1.append(f"  {tag.upper()} expected {expected[i1:i2]!r} actual {actual[j1:j2]!r}")
out1.append(
    f"  {equal_blocks} equal blocks, {equal_lines} lines, each also compared with cmp; "
    f"{len(expected)} expected lines, {len(actual)} actual lines"
)

out1.append("")
out1.append("(b) moved units: PRE text against POST text, compared with cmp")
moved = [k for k in pre_units if post_units_file.get(k, PARENT_RS) != PARENT_RS]
moved_ok = 0
reformatted = []
for key in moved:
    pu = pre_units[key]
    a, b = splitlib.span(pre_dispatch, pu)
    pre_text = pre_dispatch[a : b + 1]
    rel = post_units_file[key]
    pl = post_lines(rel)
    qa, qb = splitlib.span(pl, post_units[key])
    post_text = pl[qa : qb + 1]
    if tests_mode:
        post_text = [("    " + l) if l else l for l in post_text]
    notes = []
    if pu["vis"] != post_units[key]["vis"]:
        # Undo only the visibility change, on the declaration line.
        decl = re.compile(r"^(\s*)(pub(\([a-z:_ ]+\))? )?(?=(async |unsafe )?(fn|const|static|struct|enum|type|mod|use|trait) )")
        pre_text = [decl.sub(r"\1", l, count=1) for l in pre_text]
        post_text = [decl.sub(r"\1", l, count=1) for l in post_text]
        notes.append(f"visibility {pu['vis']} -> {post_units[key]['vis']}")
    ok, msg = cmp_bytes("\n".join(pre_text) + "\n", "\n".join(post_text) + "\n", key)
    if ok:
        moved_ok += 1
        if notes:
            out1.append(f"  OK {key} -> {rel} ({b - a + 1} lines; {'; '.join(notes)})")
    elif squash(pre_text) == squash(post_text):
        reformatted.append(key)
        out1.append(f"  WHITESPACE-ONLY {key} -> {rel}: {msg} ({'; '.join(notes + ['rustfmt reflow, the trailing comma of a signature ignored'])}; tokens are instrument 2's, literal values instrument 3's)")
    elif key in expected_edits:
        out1.append(f"  DIFFERS (declared edit) {key} -> {rel}: {msg}; the edit:")
        for d in difflib.unified_diff(pre_text, post_text, "before", "after", lineterm="", n=0):
            out1.append("    " + d)
    else:
        failures.append(f"I1b {key}: {msg}")
        out1.append(f"  FAIL {key} -> {rel}: {msg}")
out1.append(f"  {moved_ok} of {len(moved)} moved units byte-identical (visibility edits listed above), {len(reformatted)} differing in whitespace only")

# ---------------------------------------------------------------- instrument 2
out2 = []
common = [k for k in pre_units if k in post_units]
only_pre = [k for k in pre_units if k not in post_units]
only_post = [k for k in post_units if k not in pre_units]
tok_ok = 0
for k in common:
    if pre_units[k]["toks"] == post_units[k]["toks"]:
        tok_ok += 1
    elif drop_trailing_commas(pre_units[k]["toks"].split("\x01")) == drop_trailing_commas(post_units[k]["toks"].split("\x01")):
        tok_ok += 1
        out2.append(f"  TRAILING-COMMA-ONLY {k}: identical once the comma before the `)` closing its signature's parameter list is dropped (rustfmt's vertical layout)")
    elif k in expected_edits:
        out2.append(f"  DIFFERS (declared edit) {k}")
    else:
        failures.append(f"I2 token stream differs: {k}")
        out2.append(f"  DIFFERS {k}")
    if pre_units[k]["vis"] != post_units[k]["vis"]:
        out2.append(f"  visibility {k}: {pre_units[k]['vis']} -> {post_units[k]['vis']} ({post_units_file[k]})")
for k in only_pre:
    out2.append(f"  ONLY-PRE {k}")
    if not k.startswith("use "):
        failures.append(f"I2 unit vanished: {k}")
for k in only_post:
    out2.append(f"  ONLY-POST {k} ({post_units_file[k]})")
out2.append(
    f"{tok_ok} of {len(common)} units present on both sides have identical token streams "
    f"({len(moved)} of them moved); {len(only_pre)} only before, {len(only_post)} only after"
)

# ---------------------------------------------------------------- instrument 3
out3 = []
lit_ok = 0
control_bad = 0
total = 0


def lexer_count(lines, u):
    return rustlex.count_literals("\n".join(lines[u["first"] - 1 : u["last"]]) + "\n")


for k in common:
    pu, qu = pre_units[k], post_units[k]
    c_pre = lexer_count(pre_dispatch, pu)
    c_post = lexer_count(post_lines(post_units_file[k]), qu)
    control = pu["decodable"] == c_pre and qu["decodable"] == c_post
    same = pu["lits"] == qu["lits"]
    total += pu["decodable"]
    if not control:
        control_bad += 1
        failures.append(f"I3 control {k}: tool {pu['decodable']}/{qu['decodable']} lexer {c_pre}/{c_post}")
        out3.append(f"  CONTROL-MISMATCH {k}: tool pre/post {pu['decodable']}/{qu['decodable']}, lexer pre/post {c_pre}/{c_post}")
    if same:
        lit_ok += 1
    elif k in expected_edits:
        out3.append(f"  DECODE-DIFFERS (declared edit) {k}")
    else:
        failures.append(f"I3 literals differ: {k}")
        out3.append(f"  DECODE-DIFFERS {k}")
out3.append(
    f"{lit_ok} of {len(common)} units decode to identical literal lists; "
    f"{total} decodable literals (string, byte string, char, byte, doc line) on the PRE side; "
    f"{control_bad} units where the tool's count and the independent lexer's disagree"
)
# whole-file multiset
def all_lits(units):
    out = []
    for u in units.values():
        if u["lits"]:
            out.extend(u["lits"].split("\x01"))
    return out


from collections import Counter

pre_c = Counter(all_lits(pre_units))
post_c = Counter(all_lits(post_units))
out3.append(f"whole-file literal multiset, PRE {PARENT_RS} against POST {PARENT_RS} + new files:")
for lit, n in sorted((pre_c - post_c).items()):
    out3.append(f"  only PRE  x{n}: {lit}")
for lit, n in sorted((post_c - pre_c).items()):
    out3.append(f"  only POST x{n}: {lit}")

for name, body in (("instrument1", out1), ("instrument2", out2), ("instrument3", out3)):
    with open(f"{out_prefix}-{name}.txt", "w") as f:
        f.write("\n".join(body) + "\n")
with open(f"{out_prefix}-verdict.txt", "w") as f:
    f.write("FAILURES:\n" + "\n".join(failures) + "\n" if failures else "PASS: no failures\n")
print(open(f"{out_prefix}-verdict.txt").read())
sys.exit(1 if failures else 0)
