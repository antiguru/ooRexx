# Phase 6.1 final fix report

Status: BLOCKED. Finding 1 took the controller's fallback (refusal, `6bc2df715`). Finding 3 is committed (`c10b30ff1`), but it puts parse at +1.1984% against the +1.1979% limit, and the controller has the cgdiff. The gate run and the gate record section wait on that ruling.

## Commits

* `250176706` Finding 1: a STRING answer at a non-target builtin argument.
* `34501b269` Perf fix 1 (ruled): identity early return and a cold table lookup in the 88.909 helpers.
* `7c203a8ff` Perf fix 2 (ruled): the nil positions on each `Builtin` row, and a `substituted` flag on `Args` that every helper tests first.
* `384936cc9` Perf fix 3 (ruled): nil membership as a closure that only `required_string_dispatch` evaluates.
* `7eb7416c7` Finding 2 queue file, **and Minor 5** (spec section 2 and Task 12 report l.34 now state 472). A failed `git add` of the Minor 5 paths had staged them anyway, and they went into this commit. History was not rewritten.
* `c51a367df` Finding 4 queue file.
* `c92f18d51` Minor 6 queue file.
* `6bc2df715` Finding 1 fallback (ruled): the code of the four commits above is reverted by a reverse patch (rexx-exec/src has an empty diff against `821e68087`). Deviation 30 and R11 name every builtin argument position as refused, citing the review's table. The positions file asserts 66 refusals, each with the oracle's error in its comment, and 4 oracle-identical direct-`.nil` rows. The cold-site design was not tried: the oracle orders the builtin's own 40.12 (`changestr('a','a',o,'x')`) and STRIP's 93.915 option check ahead of the 88.909, and conversion runs before both.
* `c10b30ff1` Finding 3: `ArraySlots`, plus the row 33 and 11a corrections.

## Finding 1

* Shared site: `Interp::required_string_arguments` (`dispatch/reqstr.rs`). The positions listed by `crate::builtin::nil_argument_positions` convert through `required_string_or_nil`. Every other position keeps `required_string_value` and its refusal.
* The table `NIL_ARGUMENT_POSITIONS` (`builtin.rs`) has one row per position in the review's table. Each row is `Checked`, where the builtin's own 40.x or 93.904 check raises (`whole_number` names `Args::object`, `pad_byte` names the converted `.nil`), or `MethodString(n)`, where `method_string_arguments` / `method_string_argument` raise 88.909 numbered as the forwarded `String` method numbers it. The helper runs in each of the bodies that forward (CHANGESTR, COUNTSTR, POS, LASTPOS, OVERLAY, INSERT, TRANSLATE, VERIFY, STRIP, COMPARE, ABBREV, WORDPOS, the bit family, DATATYPE). It runs after the builtin's own checks and before the method's others. STRIP checks its set after its option, as `RexxString::strip` does.
* `Args::no_string_value`: the converted value is `.nil` and the original is not. That tells this case apart from a `.nil` the program passed itself.
* Witness: `tests/string_answer_arguments.rs` with `tests/string_answer_arguments/positions`. It holds 49 positions with `.object~new`, 11 with `.directory~new`, and 9 ordering and direct-`.nil` cases (armed). It also holds 1 unarmed program and 3 target positions that refuse. Every non-refusal expectation is the oracle's stderr, path-normalised, identical across 2 runs. REWRITE=1 left every oracle row unchanged.
* Negative controls: forcing `required_string_arguments` to never take the nil arm reddens the test. Forcing `no_string_value` to false reddens it too. Both were restored and checked with grep and `git diff`.
* Deviation 30 (`phase-4-exclusions.txt`) and spec R11 state the split. The Deviation 30 table gained the measured target rows: CHANGESTR/COUNTSTR/STRIP SIGSEGV, `pos('z', o)` 1190, `abs`/`sign`/`trunc` 93.943. Each was re-run on the oracle.
* `refusal-sites.tsv`: no derived row moved, and `refusal_sites` passed.
* Per-step check at the finding 1 tree:
  * fmt.
  * `cargo clippy -p rexx-exec --all-targets --features pinning,sharing -- -D warnings`: exit 0.
  * `cargo test -p rexx-exec --no-fail-fast`: exit 0, 1990 passed, 0 failed, 1 ignored.
  * Corpus pair (`REXX_CORPUS_GATE=1 --test corpus --test ir_recorded_oracle`): exit 0, 29 passed 1 ignored, 21 passed.

### Perf: FAILED, four measurements

`callgrind.sh -r 2` over all programs, base61 against `250176706` (scratchpad `perf/cg-f1.log`, exit 0, spreads 0.0002% at most):

| program | d% vs base61 |
|---|---|
| strings | +31.6095 |
| alloc4c | +7.7275 |
| parse | +4.2050 (limit +1.1979) |
| decrender | +3.5746 |
| rexxcps | +1.3272 |
| others | equal to `cg-close2`'s close column to 4 places |

Cause, first reading: `method_string_arguments` searched the table by name on every call. That was incomplete. Under an armed latch every heap string argument builds the converted vector, so the converted path is the common one.

All runs below are `callgrind.sh -r 2` over all programs against base61, with binaries built from a `git archive` of the commit. Each archived commit passed fmt, clippy with pinning,sharing, `cargo test -p rexx-exec` and the corpus pair, all exit 0.

| program | close | 250176706 | 34501b269 | 7c203a8ff | 384936cc9 |
|---|---|---|---|---|---|
| strings | +0.3735 | +31.6095 | +32.5161 | +3.8803 | +2.8026 |
| parse | +1.1979 | +4.2050 | +4.2050 | +1.4843 | +1.5884 |
| alloc4c | -0.0654 | +7.7275 | +7.7275 | +0.4437 | +0.6346 |
| decrender | -1.0609 | +3.5746 | +3.5745 | -0.7581 | -0.6446 |
| rexxcps | +0.2694 | +1.3272 | +1.3272 | +0.4410 | +0.4426 |

At `384936cc9` the cgdiff from close on strings is `builtin::run` +414,000,025 self Ir. Per line (`perf/strings-run-lines-f1d.txt`):
* +189M on the `substituted |= read == ObjRef::NIL && ...` line, where `ObjRef` equality decodes the tag.
* +144M on the call with the extra argument.
* +108M on the wider `Args` construction.

## Finding 3 (uncommitted)

* `rexx-core/src/slots.rs`: `ArraySlots`, the slots plus a kept 1-based last-item index (`ArrayClass::lastItem`). Reads go through `Deref<Target = [Option<ObjRef>]>`. There is no `DerefMut`, so every write goes through a method that maintains the index. The methods are `set`, `set_within`, `fill`, `overwrite`, `push`, `pop`, `insert`, `remove`, `resize`, `clear` and `replace`. `last_item()` debug-asserts the cached index against a recomputation. `Body::Array.slots` is now `ArraySlots`.
* Site enumeration. Before the change: `grep -rn "Body::Array\|Body::array(" crates --include=*.rs`, 94 lines (scratchpad `f3/grep-body-array.txt`). After the type change, the compiler listed every write and construction site (scratchpad `f3/check1.txt`):
  * Writes:
    * `dispatch/collection.rs` `write_slot` and `array_grow`
    * `dispatch/array.rs` `array_resize`, the reshape `*slots = grown` and `native_array_put`
    * `dispatch/array/sort.rs` `write_back`
    * `dispatch/array/surface.rs` `clear_array_slot` and fill
    * `dispatch/collection/list.rs` put
    * `dispatch/collection/queue.rs` put
    * `dispatch/hash.rs` `write_slot`
    * `rexx-core` bench and test sites
  * Unchanged writes that keep compiling against the new methods: `array_splice` remove, `array_splice_slot` insert, surface `pop`, list `clear`, and the message parties `push`.
  * Constructions: `state.rs`, `array.rs` (new, of, multidimensional), `buffer.rs`, `context.rs` (twice), `library.rs`, `library/surface.rs`, `object_protocol.rs` (twice), `string.rs`, `tests.rs` (twice), `rexx-api` fake and tests.
* `append_slot` reads `slots.last_item()` instead of scanning.
* Tests:
  * `slots::tests::every_write_keeps_the_last_item`.
  * `dispatch::tests::append_to_a_dense_array`, `append_after_a_sparse_put_and_empty`, `append_after_a_sized_new_and_empty`, `append_after_a_sparse_put` and `append_after_appends_and_empty`. Each asserts the oracle's answers, measured at 2,000 and 200,000 twice each. In release each runs 200,000 appends with a 5 s bound.
  * `append_after_every_kind_of_write`: the oracle's output, 2 of 2.
* All pass in debug and release (release 0.10 s).
* Negative control: restoring the scan in `append_slot` fails three shape tests in release at 21.7 to 22.2 s. Shape d is not slow under the scan, because its last item is at the end. The review's d figure was cumulative `time('e')`. The fix was restored and checked with grep.
* Inverting `wrote()`'s clear branch reddens `append_after_a_sparse_put_and_empty`, `append_after_every_kind_of_write` and `append_after_appends_and_empty` in debug, each with "the cached last item disagrees with the slots". The branch was restored and checked with grep.
* Task 12 report row 33 and the 11a report's Step 7 now state the trailing-empty-slot case (working tree).
* Per-step check on the working tree:
  * `cargo test -p rexx-exec -p rexx-core -p rexx-api --no-fail-fast`: exit 0, 2332 passed, 0 failed, 1 ignored.
  * Corpus pair: exit 0.
  * Clippy first failed on `Instant::now` in `append_shape`. It is now under `#[expect(clippy::disallowed_methods)]`, and the rexx-exec (pinning,sharing) and workspace clippy runs then finished clean.
* Per-step check on archived `c10b30ff1`:
  * fmt 0, clippy pinning,sharing 0, workspace clippy 0.
  * `cargo test -p rexx-exec -p rexx-core -p rexx-api --no-fail-fast`: exit 0, 2332 passed, 0 failed, 1 ignored.
  * Corpus pair: exit 0, 29+1 ignored and 21.
  * Release append tests: 6 passed in 0.10 s.
* Perf (`perf/cg-f3.log`, `-r 2`, base/close/fb/f3), d% vs base61:
  * fb equals close everywhere.
  * f3:
    * parse +1.1984, against close +1.1979 and the +1.1979 limit.
    * heapshape +0.4858 (close +0.0093).
    * alloc +0.4253 (close -0.0318).
    * dirread +0.1582 (close +0.1049).
    * pingmsg +0.0537 (close -0.0421).
    * Every other program is about +7,500 Ir, fixed.
  * Nothing newly exceeds +0.5%.
  * The script exits 1 on a SPREAD flag for base61's pingsem cell (0.0127%).
  * cgdiff fb→f3:
    * heapshape: `native_array_new` +7.0M (`ArraySlots::new` scans an empty `new(n)`), `native_array_put` +4.0M.
    * alloc: `from_utf8` +66M, memcmp +24M, `native_array_of` +27M.
    * parse: hash `install_store`/`insert_in`/`write_slot` +6.1K.
* Not yet done: the controller's perf ruling, the gate run and the gate record section.

## Concerns

1. Minor 5 shares commit `7eb7416c7` with the finding 2 queue file.
2. The spec section 2 paragraph still carries 520 and 528 and "Ruling R4 keeps 512", all measured against 512. The R4 row still reads "== 512 Kept". The brief named only the first sentence.
3. Builtin positions outside the review's table still refuse even where the oracle may be defined: RXQUEUE 2 (40.26), the stream builtins (93.938), D2X 1 (93.928), BITOR and BITXOR 2, MAX 3 and beyond. These were not measured per position here.
4. New, pre-existing divergence, not queued: `b = .array~of('c','a','b'); b~remove(3); b~sort`. The oracle sorts, rc 0. This crate raises 98.975 "Missing array element at position 3", the same before and after finding 3.
