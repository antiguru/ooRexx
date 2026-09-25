# Re-review: Surface Task 3, fix round 2 (`6c96144d8..a442e0b5b`)

VERDICT: **APPROVED**

Independent verification, not just prose reading: extracted `6c96144d8`
(base) and `a442e0b5b` (head) with `git archive` into this session's
scratchpad, each with its own `CARGO_TARGET_DIR`, and ran, from a clean
checkout on each side:

* `cargo fmt --all --check` on head: exit 0.
* `cargo clippy -j 4 -p rexx-core -p rexx-exec -p rexx-api --all-targets -- -D
  warnings` on head: exit 0.
* `cargo test --release -p rexx-api --test values`: **83 passed, 0 failed on
  both** base and head -- matches the report's claim exactly.
* `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus
  corpus_differential`: **604 of 604 matching on both** base and head. This
  independently confirms nothing observable moved, on the corpus as a whole
  and specifically for `library_native_object_arguments.rex`.

Scratch target directories deleted by path after.

## Item 1 -- `dispatch/library.rs:247-251`

**Closed**, and checked past the example given. `Refused::NotLogical`'s
`found` field is `logical()`'s `Err` value (`library.rs:473-484`), which is
either the string `native_string_conversion` produced, or -- for a
`Decoded::SmallInt` outside `{0,1}`, which short-circuits before any
conversion runs (`t~logical(2)` is line 29 of the corpus program itself) --
the raw, unconverted argument. In that second case `native_found` computes
the argument's own `stringValue()`, the same mechanism the arms above use, so
the sentence is imprecise about *how* the string is produced for that one
path. It is not observably wrong: an integer's own `stringValue()` and its
string conversion render the same digits, so `found` is correct either way,
and the wording matches the corpus program's own pre-existing header ("except
a logical's, which is the string the conversion answered", untouched by this
diff) that the brief explicitly told the implementer to align with. Not
flagging as a defect; noting it was checked rather than skipped, since the
brief called this arm out for extra scrutiny.

## Item 2 -- `docs/superpowers/plans/phase-4-exclusions.txt:5103-5107`

**Closed.** All three named files
(`lang/library_native_integer_arguments.rex`,
`lang/library_native_object_arguments.rex`,
`lang/library_native_special_arguments.rex`) exist at those paths, plus Task
2's `lang/library_routine_argument_errors.rex`. The bare count is gone, no
new count introduced.

## Item 3 -- `rust/corpus/phase-8.txt:205-214`

**Closed.** The false "shipped extension" / "nothing shipped" split is
replaced by a derivable rule ("witnessed as far as its entry point runs")
plus the two named exceptions, `TestMutableBufferLength` and
`TestVariableReferenceValue`. Verified against `layout.rs`: unlike
`MutableBufferData`/`SetMutableBufferCapacity` (declared `aborts`, reaching
`layout::abort`, rc 134 per the report), `MutableBufferLength` and
`VariableReferenceValue` are plain `call` entries that produce the "Phase 8
loud refusal" (rc 120) the report describes -- "an API function this crate
answers loudly" is accurate for both, and both names are real declarations in
`testbinaries/orxmethod.cpp`.

## Item 4 -- `rust/crates/rexx-exec/src/lib.rs:5694`

**Closed.** The unenumerable negative clause ("and nothing else does") is
deleted; the sentence left behind states only what the walk does.

## Item 5 -- the duplicated rule, `NativeState::renders_its_own_string_value`

**Closed**, verified by re-deriving the equivalence rather than trusting the
report:

* `NativeState` has exactly three variants (`Buffer`, `Stream`, `Pointer`,
  `body.rs:153-166`); the new method's match (`Buffer | Pointer => true,
  Stream => false`) is exhaustive over them, so a fourth variant is a compile
  error at this decision, as intended.
* `native_found`'s old guard, `state.buffer().is_none() &&
  state.pointer().is_none()`, is true only for `Stream`; `!renders_its_own_
  string_value()` is true only for `Stream`. Same.
* `redirect_of`'s old guard, `state.buffer().is_some() ||
  state.pointer().is_some()`, is true for `Buffer` or `Pointer`;
  `renders_its_own_string_value()` is true for the same two. Same. Both call
  sites use the new method in the polarity that reproduces the old boolean.
* The doc comment states what the method answers (with citations to the two
  C++ `stringValue` overrides), not how either caller uses it.
* Searched every remaining `.buffer()`/`.pointer()` use in the tree
  (`value.rs:334,418,590,598,668`, `library.rs:363,544`, `dispatch.rs:8570`,
  `invoke.rs:901`, two test files): all of them extract the held buffer or
  address for further use, none re-derive the "renders its own string value"
  boolean by another route.
* Behaviour proven unchanged independently (see the top of this report), not
  just by trusting the report's own before/after run.

## Item 6 -- `docs/superpowers/plans/phase-4-exclusions.txt:5089-5095`

**Closed.** The five names now recorded (`address_with_stream`,
`executable_context`, `sys_file_functions`, `security_manager`,
`call_miss_not_cached`) match exactly what the round-1 re-review measured and
what this round's own Control C9 reports. No new count introduced.

## Item 7 -- `rust/crates/rexx-exec/src/value.rs:999-1006`

**Closed.** The stale head ("A buffer's own body holds the text, named or
not.") is deleted; the surviving comment is the one the arm's guard actually
justifies (`state.renders_its_own_string_value()`, covering buffer and
pointer). Grepped the tree for any other stray copy of that stale sentence:
none found. Did not find a second doubled-header arm anywhere in the
round's touched files.

## Item 8 -- the report's own gate_table_c sentence

**Closed.** `task-3-report.md`'s "Fix round 2" section now attributes the 82
rows correctly: File instance 50, Stream instance 24, StreamSupplier instance
8, all Phase 7; Alarm and Ticker are Phase 6 and unset (not gated by that
check), matching the round-1 re-review's own count.

## Constraints checked

No em-dash in any added line (`git show a442e0b5b` filtered for the
character: none). No comment states a set's cardinality (item 2's count was
removed, no new count added anywhere in the diff). No `unsafe` touched. The
one code change is behaviour-preserving, confirmed by an independently
rebuilt `604 of 604` corpus match on both sides and identical `values.rs`
pass counts, not merely the report's own say-so.

## New findings

None that block. The one thing worth a sentence: item 1's comment is
imprecise about mechanism for the `SmallInt` short-circuit path (see above),
but it is not false about what gets displayed, it mirrors already-approved
corpus prose, and the brief itself directed this wording -- not raised as a
required fix.
