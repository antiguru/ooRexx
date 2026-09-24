"""A test-module move: a parent's inline `#[cfg(test)] mod NAME { ... }`
becomes `mod NAME;` and a file of its own. Generalises move_tests_lib.py to
a module that need not close the file: the module ends at the first line
after its opening that is exactly `}` and starts outside any literal or
comment.

Every line of the module body loses its first four spaces, except a line that
starts inside a string literal: one that is not a backslash continuation, or
a continuation not starting with four spaces, stays verbatim (a continued
line's leading whitespace is skipped by Rust, so stripping it is safe; any
other in-literal byte is part of the value).

usage: move_tests_mod.py SRC_PARENT_RS OUT_PARENT_RS OUT_CHILD_RS NAME OUT_REMOVED_JSON
"""
import json
import os
import sys

import rustlex

src_path, out_parent, out_child, name, out_removed = sys.argv[1:6]
text = open(src_path).read()
lines = text.split("\n")
assert lines[-1] == "", "file ends with a newline"
lines = lines[:-1]
LICENSE = lines[:10]
states = rustlex.line_start_states(text)

opens = [i for i, l in enumerate(lines) if l == f"mod {name} {{"]
assert len(opens) == 1, opens
start = opens[0]
assert lines[start - 1] == "#[cfg(test)]"
end = next(i for i in range(start + 1, len(lines)) if lines[i] == "}" and states[i] is None)

body, verbatim = [], 0
for i in range(start + 1, end):
    line = lines[i]
    if line == "":
        body.append(line)
        continue
    st = states[i]
    if st in ("str", "raw", "block") or (st == "str-cont" and not line.startswith("    ")):
        body.append(line)
        verbatim += 1
        continue
    assert st in (None, "str-cont"), (i + 1, st, line)
    assert line.startswith("    "), (i + 1, line)
    body.append(line[4:])

# A body that opens with a blank line (`mod x {` then an empty line) uses
# that line as the separator after the license header, so the child never
# opens with two blank lines, which rustfmt would collapse.
head = lines[:start] + [f"mod {name};"] + lines[end + 1 :]
assert not os.path.exists(out_child), out_child
os.makedirs(os.path.dirname(out_child), exist_ok=True)
open(out_parent, "w").write("\n".join(head) + "\n")
sep = [] if body and body[0] == "" else [""]
open(out_child, "w").write("\n".join(LICENSE + sep + body) + "\n")
json.dump(list(range(start, end + 1)), open(out_removed, "w"))
print(f"moved lines {start + 2}-{end} (1-based) of {src_path}, {len(body)} lines, {verbatim} kept verbatim inside literals; {len(lines) - end - 1} lines after the module stay")
