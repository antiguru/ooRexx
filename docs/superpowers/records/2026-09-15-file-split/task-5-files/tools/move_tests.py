"""Commit 1's move: `dispatch.rs`'s inline `#[cfg(test)] mod tests { ... }`
becomes `mod tests;` and `dispatch/tests.rs`.

Every line of the module body loses its first four spaces. That is safe for
code, and safe for a line that starts inside a string literal only when the
previous line ended in a backslash continuation (Rust skips a continued line's
leading whitespace); any other in-literal line start aborts the move.

usage: move_tests.py SRC_DISPATCH_RS OUT_DISPATCH_RS OUT_TESTS_RS
"""
import sys

import rustlex

LICENSE = open(sys.argv[1]).read().split("\n")[:10]

src_path, out_dispatch, out_tests = sys.argv[1:4]
text = open(src_path).read()
lines = text.split("\n")
assert lines[-1] == "", "file ends with a newline"
lines = lines[:-1]
states = rustlex.line_start_states(text)

opens = [i for i, l in enumerate(lines) if l == "mod tests {"]
assert len(opens) == 1, opens
start = opens[0]
assert lines[start - 1] == "#[cfg(test)]"
assert lines[-1] == "}", "the module closes the file"
end = len(lines) - 1

body = []
for i in range(start + 1, end):
    line = lines[i]
    if line == "":
        body.append(line)
        continue
    assert states[i] in (None, "str-cont"), (i + 1, states[i], line)
    assert line.startswith("    "), (i + 1, line)
    body.append(line[4:])

head = lines[:start] + ["mod tests;"]
open(out_dispatch, "w").write("\n".join(head) + "\n")
open(out_tests, "w").write("\n".join(LICENSE + [""] + body) + "\n")
print(f"moved lines {start + 2}-{end} (1-based) of {src_path}, {len(body)} lines")
