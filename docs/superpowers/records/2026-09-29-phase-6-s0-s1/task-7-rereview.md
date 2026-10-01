# Task 7 fix round re-review: 863d147d1..8d26923bb

Reviewed: `d112d68f9` (I1, I2, M1, M2), `8d26923bb` (M3, perf record), against
`task-7-review.md`'s I1/I2/M1/M2/M3 and the rulings at the end of `progress.md`.

## I1 (fast path only for the handle it serves)

**Addressed.** `roots.rs:181-197` (`frame_slot`), `:205-221` (`set_frame_slot`),
`:236-246` (`clear_frame_slot`) now gate on `frame.serial == activity.fast_serial`
instead of `indirect == 0`, so a non-top frame always falls through to the `_of`
path -- there is no precondition left for a caller to violate.

`fast_serial` update sites, traced against every place `segments`/`alias_count`
can change (both private to `ActivityRoots`, so this enumeration is exhaustive):

- `push_segment` (`:382-388`): sets `top_start` then calls `refresh_fast_serial`.
- `pop_slots` (`:461-477`): decrements `alias_count` by the popped aliases, then
  calls `refresh_fast_serial`.
- `set_alias` (`:509-517`): increments `alias_count` and calls
  `refresh_fast_serial`, but only on the branch where `.replace(target).is_none()`
  -- i.e. only when a slot newly becomes an alias. There is no alias-removal path
  in this file other than `pop_slots`' truncation (grepped `aliases[` / `aliases.`
  across `roots.rs`: the only writes are `resize`, `truncate`, `splice` and this
  `replace`), so `alias_count` cannot change anywhere `refresh_fast_serial` is not
  already called.
- `grow_slots_of` (`:575-587`) updates `top_start` but not `fast_serial` --
  correctly: growing (at or below the top) changes no segment's `serial` and no
  `alias_count`, so the top's identity is unchanged and there is nothing to
  refresh.
- `take_frame_aliases`/`put_frame_aliases` (REPLY save/restore, `resume_reply`
  at `dispatch.rs:2514-2521`): the re-push goes through `push_slots` ->
  `push_segment`, already covered; `put_frame_aliases` itself only calls
  `set_alias`, already covered.
- `CallerSwap` (`surface.rs:865-876`) no longer touches `alias_count` or
  `segments` at all (see below), so it needs no refresh call and has none.

`u64::MAX` as the sentinel is safe: `next_serial` (`roots.rs:150-151`) starts at
0 and increments by 1 per `push_slots`, so reaching `u64::MAX` is not reachable
in any run this project can produce.

Traced `park_reply`/`resume_reply` (`dispatch.rs:2334-2369`, `:2506-2542`) and
`variables.rs`'s three accessors plus `variable_in` -- all consistent with the
review's caller enumeration; `variable_in` still uses `frame_slot_of` (correct,
unaffected by this round).

`segment()`'s check (`roots.rs:295-302`) is `assert_eq!` (was
`debug_assert_eq!`), and its doc and the `SlotFrame` doc were updated to say
"panics" rather than "caught (a debug assertion)" -- accurate, not overclaiming.

## begin_indirect removal

**Addressed.** `git grep -n 'begin_indirect\|end_indirect'` over the whole tree
(code and docs, excluding the review package itself) finds nothing. The only
two former call sites, `CallerSwap::new` and its `Drop` (`surface.rs:865-876`),
no longer call them.

Traced why this is correct rather than merely absent: `CallerSwap` swaps
`Activity::running`/`suspended` (not `ActivityRoots::segments`), so after the
swap the caller's `SlotFrame` is not the current top segment's record (whatever
was on top when the native call happened still is). `context_variable`/
`set_context_variable`/`drop_context_variable` (`surface.rs:343-398`) all read
`self.activation().frame` -- during a `CallerSwap` that's the caller's frame --
and pass it to `variable`/`set_variable`/`clear_variable`, which call the fast
accessors. Since the caller's `serial` cannot equal the current top's `serial`
(serials are unique per `push_slots` call), the new `frame.serial ==
fast_serial` check is false and every access falls through to the `_of` path on
its own, with no indirection counter needed. This is "correct by construction"
as claimed, not just "untested."

## I2 (native token, row<<32|id)

**Addressed**, and checked for the three decode failure modes:

- **Out-of-range / unoccupied row**: `suspended_caller` (`surface.rs:735-745`)
  does `native_handles.get(row)`, which is `None` on any row past the end --
  never a panic. Exercised by the new test's last assertion (`1 << 32 | 7` when
  only row 0 exists).
- **Stale id at a row now reused**: `.filter(|native| u64::from(native.id) ==
  frame & u64::from(u32::MAX))` rejects a match when the row's current occupant
  has a different id. Exercised by the new test: token `6` is refused once row 0
  is popped and re-pushed with id `7`.
- **No panics reachable from C**: every step is `?`-chained
  (`try_from(..).ok()?`, `.get(row)...?`, `.caller?`); nothing indexes
  unconditionally. `frame: u64` reaching this function originates from
  `Activation::set_frame`, itself fed by `Interp::native_token`, never from a
  raw C-supplied bit pattern, so the only untrusted-shaped input is staleness,
  which is handled.

`native_token` (`library.rs:1091-1101`, the encode side) does `len() - 1` and
indexes unconditionally, so it is not panic-safe against an empty
`native_handles` -- but it inherits the same precondition as the pre-existing
`native_frame()`/`native_frame_mut()` (`# Panics: As Interp::native_frame`), and
every call site (`library.rs:94,167,217,263`) runs it only after
`push_native_frame`. Not a new risk; consistent with the file's existing
pattern of unchecked internal helpers guarded by call-site discipline. Not
raised as a finding.

The new test `a_stale_native_frame_token_is_refused` (`run/tests.rs:717-752`)
does fire: traced row/id arithmetic by hand for all four assertions, matches.

## M1, M2, M3

- **M1 (`mutate-4b.sh`) addressed.** The parenthetical naming the removed
  `grow_slots` invariant text is deleted; "panics `corpus_differential`" is kept,
  one of the two options the review offered.
- **M2 (descent-assertion test) addressed, and confirmed live**, not just
  present. Traced `an_alias_that_does_not_descend_is_refused`
  (`tests/roots.rs:462-477`) by hand: aliasing `outer` (depth 0) to
  `inner`'s (depth 1) own slot makes `alias_count` 1, which routes
  `frame_slot(outer, 0)` through `resolve` -> `resolve_aliased`, whose
  loop resolves `outer`'s alias to `Target::Slot{frame: inner, ..}` and checks
  `(inner.depth, index) < (outer.depth, index)` == `(1,0) < (0,0)` == false,
  tripping the `debug_assert!` with the expected text. This is the scenario
  the original review named ("alias an outer slot to `slot_ref(inner, _)`").
- **M3 (perf verdict) addressed.** `phase-6-perf.md`'s "### Round 3" section
  ends with a verdict line naming `extcall`'s `+0.3642%` as the one program over
  budget and `assign`'s `+4.21%` as the one wall-clock bar exceeded, both
  matching the round-3 tables above it.

## Prose rule

No false or future-describing statement found in the changed comments/docs.
Specifically checked: the `SlotFrame`/`segment()` doc updates ("panics" instead
of "a debug assertion") are true post-promotion; the removed preconditions on
`frame_slot`/`set_frame_slot`/`clear_frame_slot` and the `CallerSwap` doc's
indirection sentence are deletions of statements that are no longer true, not
reworded false ones; `suspended_caller`'s doc gained "or that call has returned"
which is an accurate description of the row-reuse case. No comment states a
count of a mutable set. No instance found of a comment relocated verbatim
between two sites in this diff (the only textual move-like edit is M1's
deletion, not a relocation), so that sub-check has nothing to flag.

## New issues

None.

## Verdict

**Approved.**
