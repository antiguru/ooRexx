"""An independent count of Rust string/char/byte-string/byte literal tokens
in a source fragment, written from scratch (no syn, no proc-macro2, no
reuse of any code the Rust decoder shares) as a control: if this disagrees
with the Rust decoder's own count of the same file, one of the two has a
blind spot, which is exactly the failure mode fix round 2 found in the
first version of the decoder (it silently skipped every literal inside a
macro's arguments and still exited 0, reporting an undercount as if it
were a complete one).

Handles: line comments, block comments (Rust nests them), backslash
escapes inside normal strings/chars (so an escaped quote does not end the
literal early), and raw strings/byte-strings with a matched hash count
(r"...", r#"..."#, r##"..."##, ..., and their b-prefixed forms). Counts
tokens, does not decode them -- decoding is the Rust program's job; this
script only has to agree with it on *how many* there are.

One deliberate exception to "these are just comments": a `///` or `//!`
doc comment is lexically desugared into a `#[doc = "..."]` string literal
by rustc's own tokenizer (and by `proc_macro2`, which mirrors it), so the
Rust decoder legitimately reports one STR token per doc-comment line. A
plain `//` comment (including a `////`-or-more divider, which rustc's own
rule excludes from doc-comment treatment) contributes none. Getting this
wrong showed up immediately as a real disagreement against the Rust
decoder on `Stats.rs` (`/// Relative spread, as a fraction of the
median.` on the `spread` method) before this exception was added --
recorded rather than quietly special-cased away, since it is exactly the
kind of disagreement this control exists to surface.
"""
import sys


def count_string_char_literals(text):
    i = 0
    n = len(text)
    count = 0

    def is_ident_char(ch):
        return ch.isalnum() or ch == "_"

    def skip_line_comment(i):
        j = text.find("\n", i)
        return n if j == -1 else j + 1

    def is_doc_comment_line(i):
        # `///x` is an outer doc comment, but `////...` (a fourth slash)
        # is rustc's own carve-out back to an ordinary comment. `//!` is
        # always an inner doc comment.
        if text[i : i + 3] == "///" and text[i : i + 4] != "////":
            return True
        return text[i : i + 3] == "//!"

    def skip_block_comment(i):
        # Rust block comments nest.
        depth = 1
        i += 2
        while i < n and depth > 0:
            if text[i : i + 2] == "/*":
                depth += 1
                i += 2
            elif text[i : i + 2] == "*/":
                depth -= 1
                i += 2
            else:
                i += 1
        return i

    def raw_hash_count(i):
        """At `i`, expect 'r' already consumed by caller; text[i] may be
        '#'*k then '"'. Returns (index after opening delimiter, k) or
        (i, None) if this is not actually a raw-string opener."""
        j = i
        k = 0
        while j < n and text[j] == "#":
            k += 1
            j += 1
        if j < n and text[j] == '"':
            return j + 1, k
        return i, None

    def skip_raw_body(i, k):
        terminator = '"' + ("#" * k)
        j = text.find(terminator, i)
        return n if j == -1 else j + len(terminator)

    def skip_escaped_literal(i, quote):
        """`text[i]` is the opening quote (' or \"); returns index just
        past the matching close, honouring backslash escapes."""
        i += 1
        while i < n:
            c = text[i]
            if c == "\\" and i + 1 < n:
                i += 2
                continue
            if c == quote:
                return i + 1
            if c == "\n" and quote == "'":
                # A lone `'` that never closes on the same line is a
                # lifetime or a stray apostrophe, not a char literal.
                return None
            i += 1
        return None

    while i < n:
        c = text[i]
        if c == "/" and i + 1 < n and text[i + 1] == "/":
            if is_doc_comment_line(i):
                count += 1
            i = skip_line_comment(i)
            continue
        if c == "/" and i + 1 < n and text[i + 1] == "*":
            i = skip_block_comment(i)
            continue

        prev_is_ident = i > 0 and is_ident_char(text[i - 1])

        # b"...", b'...'
        if c == "b" and not prev_is_ident and i + 1 < n and text[i + 1] in ('"', "'"):
            end = skip_escaped_literal(i + 1, text[i + 1])
            if end is not None:
                count += 1
                i = end
                continue
        # br"...", br#"...", ...
        if c == "b" and not prev_is_ident and i + 1 < n and text[i + 1] == "r":
            after, k = raw_hash_count(i + 2)
            if k is not None:
                count += 1
                i = skip_raw_body(after, k)
                continue
        # r"...", r#"...", ...
        if c == "r" and not prev_is_ident:
            after, k = raw_hash_count(i + 1)
            if k is not None:
                count += 1
                i = skip_raw_body(after, k)
                continue
        # "...": ordinary string literal.
        if c == '"':
            end = skip_escaped_literal(i, '"')
            count += 1
            i = end if end is not None else i + 1
            continue
        # '...': ordinary char literal, distinguished from a lifetime tick
        # by whether it actually closes.
        if c == "'":
            end = skip_escaped_literal(i, "'")
            if end is not None:
                count += 1
                i = end
                continue
            i += 1
            continue
        if is_ident_char(c):
            j = i + 1
            while j < n and is_ident_char(text[j]):
                j += 1
            i = j
            continue
        i += 1

    return count


if __name__ == "__main__":
    path = sys.argv[1]
    with open(path) as f:
        text = f.read()
    print(count_string_char_literals(text))
