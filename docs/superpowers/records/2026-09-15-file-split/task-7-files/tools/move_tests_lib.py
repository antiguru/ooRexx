"""The test-module move: a parent's inline `#[cfg(test)] mod tests { ... }`,
which closes the file, becomes `mod tests;` and a file of its own.

Every line of the module body loses its first four spaces, except a line that
starts inside a string literal: one that is not a backslash continuation, or
a continuation not starting with four spaces, stays verbatim (a continued
line's leading whitespace is skipped by Rust, so stripping it is safe; any
other in-literal byte is part of the value).

usage: move_tests_lib.py SRC_PARENT_RS OUT_PARENT_RS OUT_TESTS_RS OUT_REMOVED_JSON
"""
import json
import sys

import rustlex

src_path, out_parent, out_tests, out_removed = sys.argv[1:5]
text = open(src_path).read()
lines = text.split("\n")
assert lines[-1] == "", "file ends with a newline"
lines = lines[:-1]
LICENSE = lines[:10]
states = rustlex.line_start_states(text)

opens = [i for i, l in enumerate(lines) if l == "mod tests {"]
assert len(opens) == 1, opens
start = opens[0]
assert lines[start - 1] == "#[cfg(test)]"
assert lines[-1] == "}", "the module closes the file"
end = len(lines) - 1

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

head = lines[:start] + ["mod tests;"]
open(out_parent, "w").write("\n".join(head) + "\n")
open(out_tests, "w").write("\n".join(LICENSE + [""] + body) + "\n")
json.dump(list(range(start, end + 1)), open(out_removed, "w"))
print(f"moved lines {start + 2}-{end} (1-based) of {src_path}, {len(body)} lines, {verbatim} kept verbatim inside literals")
