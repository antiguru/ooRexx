# Phase 9d Task 3 review

## Spec compliance
Compliant with one wrong row. Seeds, the seven implement rows and the probe scripts hold up. The `hash.rs:214` row says `reached = no`, and a one-line probe reaches it.

## Task quality
Needs fixes (one Important).

## Findings
* Important, `refusals.tsv` row `hash.rs:214` "a hash class ... Body::Native / any hash method": claimed unreached, but `.Message~new(.Stem, .array~of('NEW', .Class))~send~items` ends in the crate's `method "ITEMS" of class "Stem" is not implemented` (rc 120). The oracle SEGVs (rc 139) after `built Stem` on the same program. The same shape on Set, Bag, Table, Directory, StringTable, Relation, IdentityTable and Properties makes the oracle SEGV (139) and the crate answer rc 0. Not an implement row (the oracle has no answer), but `guard` and `reached = no` are both wrong. Needs an `oracle-crashes.txt` entry and a decision for the Stem refusal.
* Minor: the report says 9 guard rows, the table has 8 (package, native Directory/StringTable, condition object, `.methods`, `.routines`, subclass INIT, native hash class, fallthrough).
* Minor: the seed probe `stackframe_library_routine.rex` needs `LD_LIBRARY_PATH` on the crate side. Without it the crate exits 158 (cannot load orxfunction), which reads as a different failure. `run.sh` sets it.
* Minor: the seven implement rows reproduce on both sides, but only the route through a `Message` with an `.Object` scope is covered. No ordinary send reaches them.

## Checks run
* Seeds: `grep -rn 'Loud::native_method(' rust/crates/rexx-exec/src` gives 5 sites (`dispatch.rs:2500`, `context.rs:1180`, `object_protocol.rs:487`, `object_protocol.rs:658`, `hash.rs:214`), and each has a row.
* Implement rows: StackFrame~executable, OBJECTNAME= on `.nil`, COPY on `.nil`, a builtin class, a Rexx class, a RexxContext and `.RexxInfo` all reproduce. The crate refuses, and the oracle answers as the table states.
* Class copy: `d = Message(.Array, ('COPY', .Object))~send; d~new~class == d` prints 0, `== .Array` prints 1, `d == .Array` prints 0. This matches the report.
* Package COPY and the condition-object hash row agree on both sides, so they are correctly `guard`.
* Fallthrough `dispatch.rs:2500`: `invocable` (`dispatch.rs:2472-2500`) falls through only for a MethodId in none of natives, method_bodies, generated_methods, native_externals or library_externals. My own attempts did not reach it:
  * a native method installed via `setMethod` or `define` on a foreign receiver;
  * `Method~run` of a native method object;
  * `forward`;
  * `.Method~new` copies;
  * `::method ... abstract`;
  * `EXTERNAL 'LIBRARY nosuchlib'` and `EXTERNAL 'REXX nosuch'`, which fail at load on both sides.
  Keep `guard`.
* Hash row, my other attempts: `allocate`, a subclass whose NEW returns `self~allocate`, a subclass inheriting a mixin, `enhanced`, and a subclass that forwards INIT to Object only all agree on both sides. Only the scoped `Class~new` shape differs.
* Probe scripts: `sweep.py` and `sweep2.py` detect by stderr regex `method "..." of class "..." is not implemented`. This matches `Loud::native_method`'s format (`lib.rs:684`, checked against a crate `.err`). A loud refusal is fatal and cannot be trapped, so a SYNTAX trap cannot hide a hit. The scripts spawn one process per send. I did not re-run the counts (3564 and 22086).
* The sweeps send `Class~new` only as a plain `~new`, never with a scope override, which is why they missed the Stem shape.

## Row changes
* `hash.rs:214` "any hash method, Body::Native": change to `reached = yes`, probe `.Message~new(.Stem, .array~of('NEW', .Class))~send~items`, oracle "SEGV rc 139", disposition `phase:` or `deviation:` (oracle-crash). Not `guard`.
