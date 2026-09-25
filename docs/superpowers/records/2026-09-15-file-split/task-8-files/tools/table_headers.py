"""Every native-table header (`static|const NAME: &[(..)] = &[` with its
visibility) and the closing `];` of each, in the given files, with the
whitespace-stripped token text of the header with the name and the
visibility removed: item-tool puts neither the header nor the `];` in any
unit, so instruments 2 and 3 cannot see them.

usage: table_headers.py FILE...
"""
import re, sys
H = re.compile(r"^(pub(\([a-z:_ ]+\))? )?(static|const) ([A-Z_]+): (&\[\(.*\)\]) = &\[$")
for f in sys.argv[1:]:
    ls = open(f).read().split("\n")
    for i, l in enumerate(ls):
        m = H.match(l)
        if not m:
            continue
        j = next(k for k in range(i + 1, len(ls)) if ls[k] == "];")
        norm = f"{m.group(3)} _: {m.group(5)} = &[".replace(" ", "")
        print(f"{f}:{i + 1}: vis={m.group(1).strip() if m.group(1) else 'private'} name={m.group(4)} header-tokens={norm} closes at {j + 1} with {ls[j]!r}")
