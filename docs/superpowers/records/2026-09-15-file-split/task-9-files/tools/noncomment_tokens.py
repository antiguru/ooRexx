"""A file's text with every comment (plain, doc and block, as rustlex.py
finds them) removed, as a whitespace-separated token list; compared between
a commit's version and the working tree's, per file, to show an edit
touched comments only. Also lists how many comment spans each side has.

usage: noncomment_tokens.py REV RUST_DIR FILE...   (FILE relative to RUST_DIR)
"""
import os, subprocess, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import rustlex

def strip(text):
    out, at, n = [], 0, 0
    for kind, a, b in rustlex._scan(text):
        if kind in ("comment", "doc", "block"):
            out.append(text[at:a]); out.append(" "); at = b; n += 1
    out.append(text[at:])
    return "".join(out).split(), n

rev, rust = sys.argv[1:3]
bad = 0
for f in sys.argv[3:]:
    old = subprocess.run(["git", "-C", rust, "show", f"{rev}:rust/{f}"], capture_output=True, text=True, check=True).stdout
    new = open(os.path.join(rust, f)).read()
    (a, na), (b, nb) = strip(old), strip(new)
    same = a == b
    bad += not same
    print(f"{f}: {len(a)} non-comment tokens at {rev}, {len(b)} now; {'IDENTICAL' if same else 'DIFFER'}; comment spans {na} -> {nb}")
print("PASS" if not bad else "FAIL")
sys.exit(1 if bad else 0)
