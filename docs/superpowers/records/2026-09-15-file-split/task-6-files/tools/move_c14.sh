#!/bin/bash
# c14: Array's collection surface, from collection.rs into dispatch/array/surface.rs,
# with the last rows of collection.rs's table, which goes.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/file-split-6
R=/home/moritz/dev/repos/ooRexx-rust-rewrite/rust
N=14
$S/tools/prep.sh $N || exit 1
cd $R/crates/rexx-exec/src
python3 - <<PY || exit 1
import sys, json; sys.path.insert(0,"$S/tools")
import splitlib
us=splitlib.load_units("dispatch/collection.rs")
ls=open("dispatch/collection.rs").read().split("\n")
m1=ls.index("// ---- Array's shared collection surface ----")+1
m2=ls.index("// ---- Array's own surface ----")+1
first=["fn native_array_all_items","fn native_array_all_indexes","fn native_array_make_array","fn native_array_is_empty","fn native_array_empty","fn native_array_has_item","fn native_array_index","fn native_array_has_index","fn native_array_remove","fn native_array_remove_item","fn native_array_supplier","fn clear_array_slot"]
second=["fn single_dimension_only","fn array_end","fn native_array_first","fn native_array_last","fn native_array_first_item","fn native_array_last_item","fn array_step","fn native_array_next","fn native_array_previous","fn native_array_append","fn native_array_insert","fn native_array_delete","fn native_array_fill","fn native_array_section","fn native_array_dimensions","fn same_class_array","fn splice_absorbing_slack"]
items=["LINES:%d-%d"%(m1,m1)]+first+["LINES:%d-%d"%(m2,m2)]+second
rows=[u["key"] for u in us if u["key"].startswith("row NATIVE_METHODS/")]
assert all(r.startswith("row NATIVE_METHODS/Array/") for r in rows), rows
keys={u["key"] for u in us}
assert all(k in keys for k in first+second)
spec=dict(
 doc=["\`Array\`'s collection surface: the methods it shares with the other","collections (\`allItems\`, \`hasItem\`, \`supplier\`, ...) and its own (\`first\`,","\`insert\`, \`section\`, ...), over the Array-shaped store collection.rs keeps."],
 uses=["use super::{Arity, Cleared, Failure, Interp, NativeMethod, ObjRef};"],
 items=items, rows=rows,
 table_doc=["/// \`Array\`'s collection-surface methods, chained into \`ObjectModel::build\`","/// after its sort family."],
 table_header="pub(in crate::dispatch) const NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[")
json.dump(spec,open("$S/c$N/spec.json","w"),indent=1)
print(len(items), len(rows))
PY
python3 $S/tools/move_child_rows.py dispatch/collection.rs dispatch/array/surface.rs $S/c$N/removed.json $S/c$N/spec.json | tee $S/c$N/move.txt || exit 1
sed -i 's/^pub(super) fn native_array_delete(/pub(in crate::dispatch) fn native_array_delete(/; s/^fn native_array_delete(/pub(in crate::dispatch) fn native_array_delete(/' dispatch/array/surface.rs
