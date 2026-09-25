"""Shared helpers for the dispatch.rs split: find a unit's lines, take units
out of a file, and describe exactly which lines were taken.

A unit is one line of `item-tool`'s output: a top-level item, an `impl`
member, or a row of a native method table. Its lines run from its first
attribute or doc comment to its last line, extended upward over any plain
`//` comment lines directly above it (no blank line between), because such a
comment belongs to the item or row it sits on.
"""
import json
import re
import os
import subprocess

HERE = os.path.dirname(os.path.abspath(__file__))
ITEM_TOOL = os.environ.get("ITEM_TOOL", os.path.join(HERE, "item-tool", "target", "release", "item-tool"))


def load_units(path):
    out = subprocess.run([ITEM_TOOL, path], capture_output=True, text=True, check=True).stdout
    units = []
    for line in out.splitlines():
        key, first, last, vis, decodable, toks, lits = line.split("\t")
        units.append(
            dict(
                key=key,
                first=int(first),
                last=int(last),
                vis=vis,
                decodable=int(decodable),
                toks=toks,
                lits=lits,
            )
        )
    return units


def is_plain_comment(line):
    s = line.lstrip()
    return s.startswith("//") and not s.startswith("///") and not s.startswith("//!") or s.startswith(
        "////"
    )


def span(lines, unit):
    """0-based inclusive (start, end) of `unit` in `lines`."""
    a = unit["first"] - 1
    while a > 0 and is_plain_comment(lines[a - 1]):
        a -= 1
    return a, unit["last"] - 1


def take(path, keys):
    """Removes the units named by `keys` from the file at `path`.

    Returns (new_lines, blocks, removed) where `blocks` maps each key to its
    text (lines joined with newlines, no trailing newline) and `removed` is
    the sorted list of 0-based line indices of the ORIGINAL file that are
    gone: each unit's own lines, and for a top-level unit (one that starts in
    column one) the blank line that separated it from what came before it.
    """
    text = open(path).read()
    lines = text.split("\n")
    assert lines[-1] == ""
    lines = lines[:-1]
    units = {u["key"]: u for u in load_units(path)}
    for key in keys:
        if key.startswith("LINES:"):
            a, b = key[len("LINES:") :].split("-")
            units[key] = dict(key=key, first=int(a), last=int(b))
    missing = [k for k in keys if k not in units]
    assert not missing, missing
    removed = set()
    blocks = {}
    for key in keys:
        a, b = span(lines, units[key])
        overlap = removed.intersection(range(a, b + 1))
        assert not overlap, (key, sorted(overlap))
        blocks[key] = "\n".join(lines[a : b + 1])
        removed.update(range(a, b + 1))
        # The blank line that separated the unit from what came before it, or
        # else the one after it. With BLANK_INDENTED set (the run.rs split,
        # whose `impl Interp` members move one by one) an indented unit's
        # separator goes too; the dispatch.rs split only took indented units
        # as table rows, which have none.
        top_level = not lines[a].startswith(" ")
        if top_level or os.environ.get("BLANK_INDENTED"):
            if a > 0 and lines[a - 1] == "" and (a - 1) not in removed:
                removed.add(a - 1)
            elif b + 1 < len(lines) and lines[b + 1] == "" and (b + 1) not in removed:
                removed.add(b + 1)
    new_lines = [l for i, l in enumerate(lines) if i not in removed]
    return new_lines, blocks, sorted(removed)


def floating_lines(path, lo, hi):
    """Non-blank lines in [lo, hi] (1-based) that no unit's span covers."""
    lines = open(path).read().split("\n")[:-1]
    covered = set()
    for u in load_units(path):
        a, b = span(lines, u)
        covered.update(range(a, b + 1))
    return [
        (i + 1, lines[i])
        for i in range(lo - 1, hi)
        if lines[i].strip() and i not in covered
    ]


def write_json(path, value):
    with open(path, "w") as f:
        json.dump(value, f, indent=1)


def _signature_close(text):
    """Index of the `)` closing the parameter list of the first `fn`
    declaration in `text`, or None."""
    m = re.search(r"(?m)^\s*(pub(\([^)]*\))?\s+)?((const|async|unsafe)\s+)*fn\s+\w+", text)
    if not m:
        return None
    open_at = text.find("(", m.end())
    if open_at == -1:
        return None
    depth = 0
    for i in range(open_at, len(text)):
        if text[i] == "(":
            depth += 1
        elif text[i] == ")":
            depth -= 1
            if depth == 0:
                return i
    return None


def squash(text_lines):
    """All whitespace removed, after dropping one comma: the one directly
    before the `)` that closes a `fn` declaration's parameter list, which is
    what rustfmt's vertical layout adds when it re-wraps a signature. No other
    comma is dropped, so `(x,)` against `(x)` anywhere else still differs."""
    text = "\n".join(text_lines)
    close = _signature_close(text)
    if close is not None:
        head = text[:close].rstrip()
        if head.endswith(","):
            text = head[:-1] + text[close:]
    return "".join(text.split())


def drop_trailing_commas(tokens):
    """`tokens` without the one comma directly before the `)` that closes the
    first `fn` declaration's parameter list (outside attributes); every other
    token, every other comma included, is kept."""
    i, n = 0, len(tokens)
    while i < n:
        if tokens[i] == "#" and i + 1 < n and tokens[i + 1] == "[":
            depth, j = 0, i + 1
            while True:
                if tokens[j] == "[":
                    depth += 1
                elif tokens[j] == "]":
                    depth -= 1
                    if depth == 0:
                        break
                j += 1
            i = j + 1
            continue
        if tokens[i] == "fn":
            break
        i += 1
    else:
        return tokens
    open_at = tokens.index("(", i)
    depth = 0
    for j in range(open_at, n):
        if tokens[j] == "(":
            depth += 1
        elif tokens[j] == ")":
            depth -= 1
            if depth == 0:
                if tokens[j - 1] == ",":
                    # A punctuation token directly before the dropped comma
                    # was Joint (`+`) only because the comma followed it:
                    # `Option<ObjRef>,)` tokenizes its `>` as `>+`, where
                    # `Option<ObjRef>)` gives `>`.
                    before = tokens[: j - 1]
                    if before and len(before[-1]) > 1 and before[-1].endswith("+"):
                        before = before[:-1] + [before[-1][:-1]]
                    return before + tokens[j:]
                return tokens
    return tokens


FIELD_VIS = (["pub", "(", "super", ")"], ["pub", "(", "crate", ")"])


def _after_field_attrs(out):
    """Whether `out` ends in one or more `#[...]` attributes (a field's doc
    comment is `# [ doc = "..." ]` in the token stream) standing directly
    after `{` or `,`: the position of a field that carries a doc comment.
    Task 10: a doc-commented field widened to `pub(crate)` has its
    visibility after `]`, not after `,`."""
    j = len(out)
    seen = False
    while j > 0 and out[j - 1] == "]":
        depth, k = 0, j - 1
        while k >= 0:
            if out[k] == "]":
                depth += 1
            elif out[k] == "[":
                depth -= 1
                if depth == 0:
                    break
            k -= 1
        if k < 1 or out[k - 1] != "#":
            return False
        j = k - 1
        seen = True
    return seen and j > 0 and out[j - 1] in ("{", ",")


def strip_field_vis(tokens):
    """`tokens` without a `pub(super)`/`pub(crate)` directly after `{` or `,`,
    or after attributes standing there: a struct field's visibility, which a
    move out of the module that constructs or reads the struct has to widen.
    Nothing else is dropped."""
    out, i = [], 0
    while i < len(tokens):
        if out and (out[-1] in ("{", ",") or _after_field_attrs(out)) and any(tokens[i : i + 4] == v for v in FIELD_VIS):
            i += 4
            continue
        out.append(tokens[i])
        i += 1
    return out


FIELD_VIS_LINE = re.compile(r"^(\s+)pub\((super|crate)\) (?=[a-z_][a-z_0-9]*: )")


def strip_field_vis_lines(lines):
    return [FIELD_VIS_LINE.sub(r"\1", l) for l in lines]
