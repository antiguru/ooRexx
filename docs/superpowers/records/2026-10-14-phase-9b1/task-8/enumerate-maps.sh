#!/bin/sh
# Prints file:line<TAB>struct<TAB>field<TAB>type for every map, set or vector
# field of the long-lived interpreter tables: Interp and what it holds by value
# (the program table, the object and environment models, the libraries, the
# activity and its scheduler), Heap and RootSet, ClassRegistry and its graph,
# a package's own data, and the rexx-api handle tables. Byte buffers
# (Vec<u8>, Vec<Vec<u8>>) are left out. Run from rust/.
structs='^(Interp|Programs|ObjectModel|EnvironmentModel|Libraries|PackageTable|Activities|GuardTable|Semaphores|Activity|PackageData|Heap|RootSet|ActivityRoots|ClassRegistry|ClassGraph|ClassDef|MethodDict|Table|Released)$'
for f in \
  crates/rexx-exec/src/lib.rs \
  crates/rexx-exec/src/plan.rs \
  crates/rexx-exec/src/environment.rs \
  crates/rexx-exec/src/dispatch.rs \
  crates/rexx-exec/src/libraries.rs \
  crates/rexx-exec/src/scheduler.rs \
  crates/rexx-exec/src/guards.rs \
  crates/rexx-exec/src/semaphores.rs \
  crates/rexx-exec/src/activity.rs \
  crates/rexx-exec/src/package_state.rs \
  crates/rexx-core/src/heap.rs \
  crates/rexx-core/src/roots.rs \
  crates/rexx-classes/src/registry.rs \
  crates/rexx-classes/src/class_graph.rs \
  crates/rexx-classes/src/method_dict.rs \
  crates/rexx-api/src/handles.rs
do
  awk -v f="$f" -v keep="$structs" '
    /^(pub(\([a-z]+\))? )?struct [A-Za-z]+ *\{/ { s = $0; sub(/^.*struct /, "", s); sub(/[ <{].*/, "", s); inside = (s ~ keep); next }
    inside && /^\}/ { inside = 0; next }
    inside && /^    (pub(\([a-z]+\))? )?[a-z_0-9]+: / && /(Map|Set|Vec)</ && !/: (Vec<u8>|Vec<Vec<u8>>|std::cell::Cell<Vec<u8>>),?$/ {
      line = $0; sub(/^ +/, "", line); sub(/^pub(\([a-z]+\))? /, "", line)
      name = line; sub(/:.*/, "", name); type = line; sub(/^[a-z_0-9]+: /, "", type); sub(/,$/, "", type)
      printf "%s:%d\t%s\t%s\t%s\n", f, NR, s, name, type
    }
  ' "$f"
done
