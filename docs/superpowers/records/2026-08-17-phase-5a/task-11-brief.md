## Task 11: the Array and the Directory that 5a's own mechanisms send to

**Goal.** The collection objects this phase's mechanisms hand around, and nothing more.

**Why here.** Three later things need one: `UNKNOWN`'s second argument is an Array (Task 12),
`ExprKind::List` must become one (M4), and the bootstrap sends `DO OVER`, `[]` and `~put` (Task 23).
**Scope is bounded and the boundary is the point**: the *documented* Array and Directory method sets
are 5c's, and this task implements what 5a's own mechanisms send, listed by name.

**Build.** An Array object; `~size`, `~items`, `~at`, `[]`; `DO OVER` an Array; a Directory's `~put`,
`[]` and `~at`. **M4:** `ExprKind::List` becomes a real Array from this commit -- measured, `(1,)~size`
is `2` on the oracle and rc 120 here. The protocol behind `DO OVER` is `requestArray`, and for a
primitive-behaviour receiver the oracle takes a C++ virtual with no dispatch at all.

**Verification, runnable now.** Oracle-differential on `(1,)~size`, on `(1,2,3)~items`, on both
`DO OVER` shapes -- `do name over publicClasses` and the comma-separated expression list over a line
continuation the prologue uses, which folds into a plain Array and is a different construct -- and on
Directory `~put`/`[]`, both engines.

**The ownership move is `ExprKind::List`'s, not `LoopKind::Over`'s.** `owners.rs:251` has
`ExprKind::List` at `Owner::Phase("Phase 5")` and `("ExprKind", "List", "Phase 5")` in the pinned
table, so it is the variant that edits all five `owners.rs` items in this commit. `LoopKind::Over` is
**already** `Owner::InScope` (`owners.rs:259`) and is not in `EXPECTED_OUT_OF_SCOPE` -- measured,
`do i over 'abc'` is rc 0 `abc` on both sides today. An earlier draft named `Over` here and would have
sent the implementer looking for a move that does not exist.

**Both value kinds this task needs already exist, so the two carried constraints are conditional
rather than owed.** Measured before dispatch: `Body::Array(Vec<ObjRef>)` is `rexx-core/src/body.rs:109`
and Task 9 allocates one at `dispatch.rs:1382` under `BehaviourId::ARRAY` for `~superClasses`, with
`Primitive::Array` at `dispatch.rs:612` and the string-value arms at `value.rs:668`, `:785`, `:1048`
and `:1301`; a directory's store is `NativeObject`'s `entries` map, already carrying `.environment`
and `.local`. So this task adds *methods* over existing bodies, and `dispatch.rs` has no `("Directory",
...)` row yet at all. `rexx-core/src/body.rs:292` is `const _: () = assert!(size_of::<Body>() <= 80);`,
so a violation is a compile error rather than a test failure, and Q4's boxing rule is that a new kind
arrives boxed in its own variant unless a recorded measurement says widening is worth it. R29 is why
that rule earns its keep: an unboxed `InstanceVar` widened `Argument` from 32 to 56 bytes and cost the
`strings` axis 1.39%. **The expected outcome is that neither constraint is exercised.** If the
implementation adds no variant, the report says so in a sentence; a boxing decision is reported only
where one was actually made. An earlier draft called this "the task that adds a value kind", which
would have invited a justification for a decision the task does not reach.

**What it cannot see.** Nothing here says the Array's *documented* set answers; that is 5c's method
half and its rows stay red. Say which names this task implements so a later reader does not read the
task as covering the class.

**Done when** `(1,)~size` is 2 on both engines, both `DO OVER` shapes match, and the method rows this
task moves are the named ones and no others. Sitting required.

---

