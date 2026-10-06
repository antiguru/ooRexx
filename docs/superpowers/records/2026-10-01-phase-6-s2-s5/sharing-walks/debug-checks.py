#!/usr/bin/env python3
# Every debug-only check in the interpreter's non-test source whose text calls
# a method or function: each `debug_assert*!` invocation, and each
# `#[cfg(debug_assertions)]` item or block, printed as
# `file:line<TAB>unshared or -<TAB>calls`, where `unshared` marks a check that
# reads under `unshared!`.
# Run from rust/.
import re, subprocess
files = subprocess.run(
    ['/bin/grep', '-a', '-rlE', r'debug_assert|cfg\(debug_assertions\)',
     'crates/rexx-exec/src', 'crates/rexx-classes/src', 'crates/rexx-api/src', '--include=*.rs'],
    capture_output=True, text=True).stdout.split()
def balanced(text, start, open_ch, close_ch):
    depth = 0
    for i in range(start, len(text)):
        if text[i] == open_ch:
            depth += 1
        elif text[i] == close_ch:
            depth -= 1
            if depth == 0:
                return text[start:i + 1]
    return text[start:]
for f in sorted(files):
    if '/tests' in f or f.endswith('tests.rs'):
        continue
    text = open(f).read()
    spans = []
    for m in re.finditer(r'debug_assert(_eq|_ne)?!\s*\(', text):
        spans.append((m.start(), balanced(text, m.end() - 1, '(', ')')))
    for m in re.finditer(r'#\[cfg\(debug_assertions\)\]\s*', text):
        rest = text[m.end():]
        # A field or a statement ends at a `,` or `;` outside brackets; an item
        # or block that opens a brace first runs to the matching brace.
        depth = 0
        for at, ch in enumerate(rest):
            if ch in '([<':
                depth += 1
            elif ch in ')]' or (ch == '>' and rest[at - 1] != '-'):
                depth -= 1
            elif depth == 0 and ch == '{':
                spans.append((m.start(), balanced(text, m.end() + at, '{', '}')))
                break
            elif depth == 0 and ch in ',;':
                spans.append((m.start(), rest[:at + 1]))
                break
    for start, span in sorted(spans):
        # Drop string literals, then look for calls.
        code = re.sub(r'"(\\.|[^"\\])*"', '""', span)
        calls = sorted(set(re.findall(r'\.([a-z_][a-z0-9_]*)\s*\(', code)) |
                       set(re.findall(r'\b([a-z_][a-z0-9_]*)\s*\(', code)) - {'debug_assert', 'debug_assert_eq', 'debug_assert_ne', 'matches', 'Some'})
        if calls:
            line = text.count('\n', 0, start) + 1
            mark = 'unshared' if 'unshared!' in span else '-'
            print(f"{f}:{line}\t{mark}\t{' '.join(calls)}")
