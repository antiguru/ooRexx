STATUS: DONE

# Fix report: value.rs / stem.rs (F5 critical, F4 important)

Task: fix branch-review-value.md findings F5 (to_number panics on Body::Stem) and
F4 (bare stem read on an unset name must auto-vivify a shared Body::Stem, not a
derived Text). F1/F2/F3/F6/F7 are out of scope (run.rs, queued elsewhere).

Final permitted files (after a coordinator correction mid-task, see below):
`value.rs`, `stem.rs`, `eval.rs`, and this report. `lib.rs` and `run.rs` stayed off
limits throughout.

## Plan

1. Read value.rs and stem.rs to understand current to_number/to_text and stem read path.
2. Measure oracle behavior for the scenarios called out.
3. Implement to_number fix for Body::Stem (F5).
4. Implement auto-vivify-on-read fix for unset bare stem (F4), update stem.rs module doc.
5. Add regression tests, naming the mutation each kills.
6. Run cargo fmt / clippy / full test suite / corpus; verify still 29/29 and green.

## A file-permission conflict, raised and resolved before implementing F4

Traced the call chain for a bare stem read (`ExprKind::Stem`) before writing any
F4 code: `eval_node` (`eval.rs`) matched `ExprKind::Variable(id) | ExprKind::Stem(id)`
in one shared arm and called `self.read(code, *id)`, a function that lives in
`lib.rs` and, on an unset slot, returns a derived `Body::Text` -- discarding which
stem was read, which is exactly the bug. Neither `lib.rs` nor `eval.rs` were in my
original permitted set (`value.rs`, `stem.rs` only), so a correct fix was not
reachable inside it. Flagged this to the coordinator before writing any F4 code
rather than shipping a half fix.

Coordinator's answer: `eval.rs` granted, `lib.rs` denied (another agent was mid-edit
there for an unrelated comment; two agents in one file is a known way this project
has lost work). Also directed: split `ExprKind::Stem` out of the shared `Variable`
arm entirely, rather than sniffing a trailing dot inside `lib.rs`'s generic `read` --
a bare stem read and a simple-variable read are not the same operation (one
auto-vivifies, one never does), and the old shared arm's own comment ("not a new
operation") is exactly the claim F4 falsifies. Also asked to check, against the
oracle, whether auto-vivification-on-read is observable outside aliasing (`drop b.`
on a never-touched stem; `b.` as a `DO OVER` target), and to confirm the vivified
object's `default` is genuinely `None`, not the derived name.

## Oracle measurements

All run under `( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx FILE )`.

| Probe | Oracle | Ours (before fix) | Ours (after fix) |
|---|---|---|---|
| `a. = 5` / `say a. + 1` | `6`, rc 0 | panic, rc 101 | `6`, rc 0 |
| `say b. + 1` (b. never touched) | error 41.1 "Nonnumeric value (\"B.\")", rc 215, **plus** one stderr line before the clause echo: `*-* Compiled method "+" with scope "String".` | panic, rc 101 | error 41.1 "B.", rc 215, **without** that stderr line -- see correction below |
| `q. = 'wd'` / `say q. + 1` | error 41.1 "Nonnumeric value (\"wd\")", rc 215 (same extra stderr line) | panic, rc 101 | error 41.1 "wd", rc 215 (same missing line) |
| `a.=5; b.=a.; say b.+1` (double hop through an aliased stem) | `6`, rc 0 | panic, rc 101 | `6`, rc 0 |
| `b.=a.` (a. never touched) / `a.1=5` / `say b.1` | `5` | `A.` | `5` |
| ...`say b.7` | `A.7` | `A.` | `A.7` |
| `drop b.` (b. never touched) / `say b.` | `B.` | `B.` (unaffected either way) | `B.` |
| `say b.` / `drop b.` / `say b.` (read before drop) | `B.` / `B.` | n/a (untested before fix; behaviourally can't differ, see below) | `B.` / `B.` |

**Correction (post-review): the `say b. + 1` / `say q. + 1` rows do NOT match
exactly.** An earlier version of this report said they did. rc and both error
lines (`Error 41`, `Error 41.1`, including the quoted nonnumeric value) match;
the oracle's full stderr for `say b. + 1` is:

           *-* Compiled method "+" with scope "String".
         1 *-* say b. + 1
    Error 41 running .../f5_b.rex line 1:  Bad arithmetic conversion.
    Error 41.1:  Nonnumeric value ("B.") used in arithmetic operation.

and ours omits the first line. That line is method-dispatch bookkeeping
(the oracle compiling and caching a method for `+` against the operand's own
class), which needs the object model 4a does not have -- filed as a KNOWN GAP
in `docs/superpowers/plans/phase-4-exclusions.txt`, owned by Phase 5. It is a
consequence of this fix, not a regression: before the fix, a bare-stem
operand hit `to_number`'s `unreachable!` and aborted before the oracle's line
could even be compared, so the divergence was unreachable, not absent. `say
'abc' + 1` (measured, no stem involved) still matches byte for byte, which is
what pins the gap to a stem's now-real object identity rather than to
anything this fix touched in the error path itself.

The last two rows answer the coordinator's "observable outside aliasing?" question:
no. `stem_drop`/`replace_stem` always allocates a fresh `Body::Stem{default:None,
tails:{}}` and rebinds the slot regardless of what was there before (`stem.rs`,
unchanged by this fix), so whether a prior `say b.` auto-vivified the slot first
makes no difference to what `drop b.` leaves behind. For `DO OVER`, the answer is
by construction: `run.rs`'s `LoopKind::Over` arm detects a bare `ExprKind::Stem`
target *syntactically*, before ever calling `eval` on it (`matches!(target.kind,
ExprKind::Stem(_))`, existing code, Deviation 1, unrelated to this fix) -- so a
stem target's read is never reached at all, and this fix cannot change that path's
behaviour either way. Confirmed by inspection, not by adding a new test, since the
guard already has direct coverage (`do_over_a_stem_target_takes_the_loud_path`,
`do_over_a_parenthesised_stem_target_is_also_caught`, existing tests, unmoved).

The second-to-last table row's mutation-catching value is folded into the new
`reading_an_untouched_stem_auto_vivifies_a_shared_object` test's `b7` assertion
(see below): if `read_stem`'s vivified object carried `default: Some(<derived
name>)` instead of `default: None`, `b7` would resolve through that wrong default
(rendering `A.` for every unresolved tail) instead of deriving its own name
(`A.7`) -- which is the coordinator's second check, confirmed by construction and
pinned by that assertion.

## F5 fix

`to_number` (`value.rs`) gained the same stem redirect `to_text` already had:
`Body::Stem { default: Some(d), .. }` recurses through `to_number(d)` (borrow on
`self.heap` dropped before the recursive call, mirroring `to_text`'s own shape
exactly); `default: None` parses the object's own `name` and reports `NotNumeric`
on failure, same fallback `to_text` renders. No new cache field: unlike
`Body::Text`'s `num`, `Body::Stem` has nowhere to memoise a parse, and a stem's
derived name (`Q.`) essentially never parses as a number, so there is nothing
costly being reparsed on the common path.

## F4 fix

Split `eval_node`'s `ExprKind::Stem` arm out of the shared `Variable` one
(`eval.rs`). `Variable` still calls `lib.rs`'s `read`; `Stem` now calls a new
`stem.rs` function, `read_stem`, which returns whatever is already in the slot,
or on a miss allocates `Body::Stem { name, default: None, tails: {} }`, binds it
into the slot, and returns it -- so the object a bare stem read hands back is the
one aliasing later mutates through, exactly matching the oracle's
`createStemVariable` firing on any variable-dictionary miss, reads included
(`VariableDictionary::getStemVariable`, `VariableDictionary.hpp:178-183`).

`read_by_name` (the function `tail_key` uses to resolve a tail *piece* variable's
value) keeps its old behaviour unchanged -- it derives a plain `Body::Text` on a
miss and never vivifies -- because a tail piece is always a plain variable
(`compound_parts`/`Tail::Variable`'s own contract), never a stem, so there is
nothing to alias. Its doc comment used to claim it also served the bare-stem case;
that claim is now false (the bare-stem case has its own function) and the comment
is corrected.

`stem_get` (compound-tail reads, e.g. `a.5` on an untouched `a.`) is **unchanged**
and correctly so: a compound read's result is always a plain value read out of (or
derived for) a tail, never the stem object itself, so it has no identity for a
later alias to reach regardless of whether the stem exists. Checked by reasoning,
not just left alone: the aliasing hazard F4 is about is specific to a *bare* stem
read handing back the object itself, which only `ExprKind::Stem` does.

### Module doc correction (stem.rs)

The old doc claimed a bare, unset stem read "is not a new operation" and needs "no
`Body::Stem` ... allocated," reasoning from two rendering-only measurements
(`say never_touched.5`, `drop x.`). Both measurements are still true and still
cited: rendering an unset stem gives the same derived name whether or not an
object was actually allocated, so neither probe could have distinguished the two
implementations. What the claim missed, stated in the correction: aliasing is
where an object's identity becomes observable, and rendering alone can never reach
it; reaching it needs a read whose *result* is stored and later mutated through a
second name, and that needs no `PROCEDURE EXPOSE` at all for a bare stem (`b. =
a.` is a plain top-level assignment) -- so this was reachable in pure 4a all
along, not gated on 4b.

## Tests added, and the mutation each kills

`value.rs` (all direct `Body::Stem` construction via `alloc_with`, matching
`stem.rs`'s own allocation shape, no activation needed):

* `to_number_on_a_stem_redirects_through_its_numeric_default` -- `a.=5; say a.+1
  -> 6`. Kills reverting the new arm back to `unreachable!`: that mutant panics
  this test instead of returning `Ok(5)`.
* `to_number_on_a_defaultless_stem_parses_its_own_name_and_fails` -- `say q.+1` on
  a stem with no default. Kills a mutation that returns some fixed `Ok` (e.g.
  `Ok(0)`) instead of parsing the object's own name and reporting `NotNumeric`.
  Also asserts rendering is untouched (`to_text` still gives `Q.`).
* `to_number_on_a_stem_aliasing_another_stem_chases_through_both` -- `a.=5; b.=a.;
  say b.+1 -> 6`, the actual aliasing shape `stem_assign` produces (`b.`'s default
  is itself a `Body::Stem`, never a copy of `a.`'s value). Kills a mutation that
  only redirects one level (e.g. matching `Body::Num`/`Body::Text` inline instead
  of recursing through `to_number` again), which would panic on the second hop.

`stem.rs`:

* `reading_an_untouched_stem_auto_vivifies_a_shared_object` -- the F4 transcript
  itself (`b.=a.; a.1=5; say b.1 -> 5; say b.7 -> A.7`). Kills two independent
  mutations: (1) reverting `read_stem` to the old derive-a-`Body::Text` behaviour,
  which would make `stem_assign`'s `is_stem` check take the "wrap as new default"
  branch and render `b1` as `A.` instead of `5`; (2) vivifying with the wrong
  default (`Some(<derived name>)` instead of `None`), which would make `b7`
  resolve through that default (`A.`) instead of deriving its own tail name
  (`A.7`) -- the coordinator's second check, pinned here.
* `an_unset_tail_piece_variable_still_derives_its_own_name_not_a_stem` -- pins
  that `read_by_name` (used for tail *pieces*, e.g. `v.i` with `i` unset) still
  takes the plain, non-vivifying path after the split. Kills a mutation that
  merges `read_by_name` and `read_stem` back into one auto-vivifying function,
  which rendering alone cannot catch (an unset piece's derived name and a fresh
  stem's derived name render identically) -- caught here instead by asserting no
  slot was bound for the piece variable at all.
* `an_untouched_stem_derives_its_own_name_with_the_period` (pre-existing, retargeted
  from `read_by_name` to `read_stem`, comment updated) -- the "measured harmless"
  half of the correction: proves the auto-vivify change is invisible to rendering
  alone, which is also why this test *couldn't* have caught F4 by itself.
* `bare_stem_assignment_shares_the_object_when_the_value_is_already_a_stem`,
  `dropping_the_whole_stem_leaves_an_old_alias_intact`,
  `reassigning_the_whole_stem_leaves_an_old_alias_intact` (pre-existing) --
  retargeted their `read_by_name` calls that were simulating a bare stem read
  (`r_value = ...`, `s_value = ...`, `a_value`/`b_bare = ...`) to `read_stem`, so
  they accurately simulate what they claim to (`u = r.`, `t = s.`, `b. = a.`),
  even though these three happened not to be able to observe the bug (the aliased
  stem was already touched in each, so `read_by_name`'s "Some" branch and
  `read_stem`'s coincide).

## Verification

* `cargo test -p rexx-exec --lib`: 175 passed, 0 failed (170 pre-existing + 5 new).
* `cargo test --workspace`: all green, no failures anywhere.
* `cargo clippy --all-targets -- -D warnings` (rexx-exec): clean.
* `rustfmt --edition 2024` on `value.rs`, `stem.rs`, `eval.rs`: no changes needed.
  `cargo fmt --check` reports one pending diff, entirely inside `run.rs`
  (`otherwise_indent` line-wrap) -- the other agent's in-progress edit there, not
  touched by this task.
* `cargo test -p rexx-exec --test corpus` and `REXX_CORPUS_GATE=1 cargo test -p
  rexx-exec --test corpus`: 29 of 29 matching, both modes.
* End-to-end through the real binary (`cargo run -q -p rexx-exec --bin rexx-run --
  FILE`), every probe in the measurement table above re-run against `ours (after
  fix)`. rc and stdout/error-message text matched the oracle on all of them,
  including the two error-path probes (rc 215, `Error 41.1` with the correct
  nonnumeric value quoted) -- but those two do NOT match exactly: the oracle's
  stderr carries one extra line ours does not, `*-* Compiled method "+" with
  scope "String".`, filed as a KNOWN GAP owned by Phase 5 (see the correction
  above and `phase-4-exclusions.txt`).

## Why these survived Task 5/9's own reviews (for the record, per the review's own framing)

F5: no corpus program does arithmetic on a bare stem, so 29/29 and 824 tests could
not see it; `to_text`'s stem arm existed and was tested, `to_number`'s never did.
F4: the module doc's own two measurements were real and correctly rendering-only;
the only way to falsify "no allocation needed" is to alias the read's result, and
aliasing an untouched stem needs no `PROCEDURE EXPOSE` for a bare stem (`b. = a.`
is plain top-level) but nothing in the 4a task's test suite happened to write that
one line.

STATUS: DONE
