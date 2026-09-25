"""A small hand-written Rust lexer, independent of syn and proc-macro2.

Two uses:

* `count_literals(text)`: how many string, byte-string, C-string, char and
  byte literal tokens `text` holds, with each `///` or `//!` doc-comment line
  counted as one (rustc desugars each into a `#[doc = "..."]` string literal,
  and the syn-based item tool therefore sees one). A control for the item
  tool's own decoded count: if the two disagree, one of them is blind.
* `line_start_states(text)`: for each line, what the lexer is inside of when
  that line starts -- None (code), "str-cont" (a normal string whose previous
  line ended in a backslash, so this line's leading whitespace is not part of
  the value), "str" (a normal string, leading whitespace IS part of the
  value), "raw" (a raw string), "block" (a block comment).
"""


def _scan(text):
    """Yields (kind, start, end) for each literal token and comment."""
    i = 0
    n = len(text)

    def ident_char(ch):
        return ch.isalnum() or ch == "_"

    while i < n:
        c = text[i]
        if text.startswith("//", i):
            j = text.find("\n", i)
            j = n if j == -1 else j
            doc = (text.startswith("///", i) and not text.startswith("////", i)) or text.startswith(
                "//!", i
            )
            yield ("doc" if doc else "comment", i, j)
            i = j
            continue
        if text.startswith("/*", i):
            depth = 0
            j = i
            while j < n:
                if text.startswith("/*", j):
                    depth += 1
                    j += 2
                elif text.startswith("*/", j):
                    depth -= 1
                    j += 2
                    if depth == 0:
                        break
                else:
                    j += 1
            yield ("block", i, j)
            i = j
            continue
        prev_ident = i > 0 and ident_char(text[i - 1])
        if ident_char(c) and not prev_ident:
            # prefixes: b", b', br", r", c", cr"
            m = i
            while m < n and ident_char(text[m]):
                m += 1
            word = text[i:m]
            if m < n and word in ("r", "br", "cr") and text[m] in '#"':
                k = m
                hashes = 0
                while k < n and text[k] == "#":
                    hashes += 1
                    k += 1
                if k < n and text[k] == '"':
                    close = '"' + "#" * hashes
                    j = text.find(close, k + 1)
                    j = n if j == -1 else j + len(close)
                    yield ("raw", i, j)
                    i = j
                    continue
            if m < n and word in ("b", "c") and text[m] == '"':
                j = _end_of_quoted(text, m, '"')
                yield ("str", i, j)
                i = j
                continue
            if m < n and word == "b" and text[m] == "'":
                j = _end_of_char(text, m)
                if j is not None:
                    yield ("char", i, j)
                    i = j
                    continue
            i = m
            continue
        if c == '"':
            j = _end_of_quoted(text, i, '"')
            yield ("str", i, j)
            i = j
            continue
        if c == "'":
            j = _end_of_char(text, i)
            if j is not None:
                yield ("char", i, j)
                i = j
                continue
            i += 1  # a lifetime or label tick
            continue
        i += 1


def _end_of_quoted(text, i, quote):
    """`text[i]` is the opening quote; index just past the closing one."""
    j = i + 1
    n = len(text)
    while j < n:
        if text[j] == "\\":
            j += 2
            continue
        if text[j] == quote:
            return j + 1
        j += 1
    return n


def _end_of_char(text, i):
    """`text[i]` is `'`. A char literal is one escape or one code point and a
    closing `'`; anything else (a lifetime `'a`, a label) is not one."""
    n = len(text)
    j = i + 1
    if j >= n or text[j] == "\n":
        return None
    if text[j] == "\\":
        k = text.find("'", j + 2)
        if k == -1 or "\n" in text[j:k]:
            return None
        return k + 1
    if j + 1 < n and text[j + 1] == "'":
        return j + 2
    return None


def count_literals(text):
    return sum(1 for kind, _, _ in _scan(text) if kind in ("str", "raw", "char", "doc"))


def line_start_states(text):
    starts = [0]
    for idx, ch in enumerate(text):
        if ch == "\n":
            starts.append(idx + 1)
    spans = [(kind, a, b) for kind, a, b in _scan(text) if kind in ("str", "raw", "block")]
    states = []
    s = 0
    for pos in starts:
        while s < len(spans) and spans[s][2] <= pos:
            s += 1
        state = None
        if s < len(spans):
            kind, a, b = spans[s]
            if a < pos < b:
                if kind == "str":
                    k = pos - 2
                    slashes = 0
                    while k >= 0 and text[k] == "\\":
                        slashes += 1
                        k -= 1
                    state = "str-cont" if slashes % 2 == 1 else "str"
                else:
                    state = kind
        states.append(state)
    return states
