#!/bin/bash
# c6: Directory/StringTable's at/put/unknown with hash_index, from dispatch.rs
# into dispatch/hash.rs after `store_entry_write`; the moved bodies' `hash::`
# qualifiers, which name the module they now sit in, dropped (declared edit).
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-6
R=/home/moritz/dev/repos/ooRexx-rust-rewrite/rust
N=6
$S/tools/prep.sh $N || exit 1
cd $R/crates/rexx-exec/src
python3 $S/tools/move_into.py dispatch.rs dispatch/hash.rs $S/c$N/removed.json 'after:fn store_entry_write' 'fn hash_index' '+fn native_hash_at' '+fn native_hash_put' '+fn native_hash_unknown' | tee $S/c$N/move.txt || exit 1
python3 - <<'PY' || exit 1
p='dispatch.rs'
s=open(p).read()
old="pub(crate) mod hash;\n"
assert s.count(old)==1
s=s.replace(old, old+"use hash::{native_hash_at, native_hash_put, native_hash_unknown};\n")
open(p,'w').write(s)
p='dispatch/hash.rs'
s=open(p).read()
a=s.index("\nfn hash_index(")
b=s.index("fn native_hash_entry(", a)
seg=s[a:b]
n=seg.count("hash::")
seg2=seg.replace("hash::owns(","owns(").replace("hash::store_at(","store_at(").replace("hash::store_put(","store_put(").replace("hash::store_entry_read(","store_entry_read(").replace("hash::store_entry_write(","store_entry_write(")
assert "hash::" not in seg2, seg2
print("qualifiers dropped:", n)
s=s[:a]+seg2+s[b:]
old="use super::{\n    Arity,"
assert s.count(old)==1
s=s.replace(old,"use super::required_string_named_argument;\n"+old)
open(p,'w').write(s)
PY
cd $R && cargo fmt --all && cargo check -q -p rexx-exec --tests 2>&1 | tail -20
