# Task 13, fix round 1 -- re-review

Scope: `d3758d520`, `ca7008069`, `df799fee0`, against F1 and F2 of
`task-13-review.md`. `fd04468f0` and `63628e8ec` are Moritz's plan-doc commits and are not reviewed
here.

## F1 -- closed

The restated command was run exactly as quoted, at `1a2ec9664`, from a temporary detached worktree
(removed afterward):

* `/bin/grep -rain 'PRIVATE ::METHOD|PRIVATE ::ATTRIBUTE' --include=*.rs --include=*.txt --include=*.tsv --include=*.rex .`
  exits 1, no output -- matches the claim that `/bin/grep` without `-E` reads `|` literally.
* `/bin/grep -rainE 'PRIVATE ::METHOD|PRIVATE ::ATTRIBUTE' --include=*.rs --include=*.txt --include=*.tsv --include=*.rex .`
  under `rust/` returns exactly three hits: `lib.rs:1592`, `lib.rs:1601`, `dispatch.rs:2522` --
  matches the restatement byte for byte.
* The same pattern run tree-wide with no extension filter additionally reaches
  `docs/superpowers/plans/phase-4-exclusions.txt:3327` and
  `docs/superpowers/plans/2026-08-17-phase-5a.md:1324` -- matches.
* `dispatch.rs:2788` at `1a2ec9664` is `assert!(stderr.contains("Error 97.1:"), "{stderr:?}");`, and
  the removed row's `"a PRIVATE ::METHOD"` string is at `:2522` -- matches the correction exactly.

No divergence from the report's restatement.

## F2 -- closed on substance, but the "nothing else" claim is false

**Both instruments exist and fire as claimed.** Verified independently (not re-verifying what the
task's own prompt already established as checked):

* `rust/crates/rexx-exec/src/lib.rs:3988`-`:4016` (`install_attribute`) confirms both code claims in
  the corpus program's comment: the `for name in names` loop calls `record_access_scope` once per
  generated name, and the setter's name is built as `let mut setter = upper; setter.push(b'=')`.
* `rust/crates/rexx-exec/src/dispatch.rs:2824`-`:2841` carries the new loud-test row (bodyless
  private accessor from an allowed caller, asserting `"a generated ::ATTRIBUTE accessor"`).
* `rust/corpus/phase-5a.txt` and `coverage.rs`'s `EXPECTED_SUBSET_5A` both gained the new program's
  entry, as claimed.

**The discriminator claim (comment's third paragraph) holds under an actual mutation.** Mutated
`install_attribute` to `continue` (skip installation entirely) for `Access::Private` attributes,
simulating "a build that dropped the accessor." Rebuilt release, ran a minimal probe
(`say .Z~b` trapped, `::ATTRIBUTE b CLASS PRIVATE` bodyless) on both engines: `CONDITION('E')` reads
`1`, not `2`, confirming the send resolves as `NoMethod` (97.1) rather than `Private` (97.2) when the
accessor genuinely does not exist. Restored the file afterward (`diff -q` clean against `git show
HEAD:...lib.rs`) and rebuilt. So `E=2` in the actual tree is a genuine signal that the accessor was
installed and privately scoped, not an artifact that would fire regardless -- the comment's claim is
true.

**The bodyless-allowed-caller gap claim (comment's fourth paragraph) is true, verified on the
oracle.** `say .K~poke` with `::method poke class / return self~a` and `::attribute a class private`
(bodyless, no `GET`/`SET`): oracle rc 0, stdout `A` (the uninitialised instance variable's name,
uppercased); this crate rc 120, `a generated ::ATTRIBUTE accessor is not implemented (Phase 5)`.
Matches the comment exactly.

**The restoration claim is right on the numbers and wrong on completeness.** Restored the retired
limb (`if attribute.access == Access::Private { Some(Loud::method_body("a PRIVATE ::ATTRIBUTE")) }`
ahead of the body-check, matching the pre-task shape at `1a2ec9664`), rebuilt release, and ran:

| instrument | result |
|---|---|
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | **159 of 160**, `[a PRIVATE ::ATTRIBUTE] lang/method_access_private_attribute.rex`, rust rc 120 vs oracle rc 159 -- matches the report exactly |
| `cargo test --release -p rexx-exec --lib` | **711 passed, 1 failed**, `a_method_body_this_crate_cannot_run_is_loud_and_its_neighbours_still_run` -- matches the report exactly |
| `cargo test --release -p rexx-exec --test collect_stress -- the_l0_subset_passes_again_under_collect_on_every_allocation` | **FAILED** -- not mentioned by the report |

The third failure is real, not a rerun artifact: it does not occur on the unmutated tree (confirmed
by reverting the mutation, rebuilding, and rerunning that single test -- `ok`, 1 passed), and it
reproduces on a second clean mutate/build/test cycle. `the_l0_subset_passes_again_under_collect_on_every_allocation`
(`crates/rexx-exec/tests/collect_stress.rs:291`) asserts a committed list of programs with zero
collections under stress-mode allocation; under the mutation,
`lang/method_access_private_attribute.rex` moves onto that list (it stops raising a full `SYNTAX`
condition through `.K~poke` and instead panics at the loud refusal before finishing its trapped
calls, so it allocates less). A full `cargo test --release --workspace --no-fail-fast` under
`REXX_CORPUS_GATE=1` confirms these are the only three test binaries that redden.

So **"the mutation reddens exactly the two instruments added for it and nothing else in the
workspace" is false as stated** -- a third, pre-existing instrument also reddens under the same
mutation. It does not undermine F2's actual closure (both sides of the access check now have a real
instrument, and both were independently confirmed to fire), but the completeness claim needs
correcting: either name `collect_stress.rs`'s reddening or drop "and nothing else in the workspace."

Restored the retired limb's removal afterward; `crates/rexx-exec/src/lib.rs` is `diff -q` clean
against `HEAD`, and the release binary was rebuilt clean before moving on.

## The `dispatchclass` axis -- sound program and instrument, one overstated ranking claim

**The program is sound.** `bench-programs/dispatchclass.rex`'s loop body is
`total = total + .Counter~bump` with `::method bump class / return 1` -- one class-method send per
pass and an accumulation, nothing else. `emptyloop.rex` (its declared "sendless control") is a bare
`nop` loop.

**The instrument is `instructions:u`, confirmed against the committed rows**, not `cycles:u`:
`bench-baselines/phase-5a-arms.tsv` line 7385 (`13-fixround-1-sendpath d3758d520 dispatchclass
pinned>changed across_builds ir small instructions:u 1.009270 ...`) and line 7384 (same, `tw`,
`1.009938`) match the report's quoted figures exactly, and the `cycles:u` rows for the same cells
(lines 7389, 7388) read a different, unquoted pair (1.037611, 1.057020) -- the report did not make
the mistake its own guidance warns about.

**The base/changed/pinned numbers all check out against the TSV exactly**: `changed`/`base` per-pass
for `dispatchclass` gives 6401.430375/6375.383679 = 1.004086 (ir) and 6482.439211/6456.448576 =
1.004026 (tw), both matching the report to six decimals; the deltas (+26.046696, +25.990635) and
`emptyloop`'s near-zero delta in the same sitting (+0.000012 ir, -0.000028 tw, both under 0.00003)
match too.

**The one false claim: "which is the second-closest cell in the file to 1% after `strings`."**
Scanning every `across_builds` `instructions:u` row in `bench-baselines/phase-5a-arms.tsv`
(`awk -F'\t' '$5=="across_builds" && $8=="instructions:u"' ... | sort -rn`), five rows from earlier
tasks are wider than `dispatchclass`'s widest reading (1.009956):

| task | commit | axis/arm | value |
|---|---|---|---|
| 11 | `f4b21eadb` | `strings`/`tw` (small and large) | 1.013710 |
| 11 | `f4b21eadb` | `alloc4c`/`tw` small | 1.011324 |
| 11 | `f4b21eadb` | `alloc4c`/`tw` large | 1.011034 |
| 9 | `4e9a0369f` | `strings`/`ir` (small and large) | 1.010616 |

All four of these are literally rows in `phase-5a-arms.tsv` and are all wider than
`dispatchclass`'s 1.009938-1.009956. So `dispatchclass` is not second in the file -- it sits behind
at least three distinct axis/arm cells from tasks 9 and 11, in addition to the current `strings`
reading (1.009685) the report names. The claim is checkable against the file as written and is
false under that literal reading. (These older cells are themselves stale in the sense that later
tasks reduced the same axes' cost -- Task 12/13's own `strings`/`ir` reading is down to 1.009685 and
`alloc4c`/`tw` down to 1.004566 -- so a charitable reading is "second among current readings," but
the sentence does not say that, and "in the file" is what a reader would go check.)

**The practical conclusion is unaffected**: `dispatchclass`'s own accumulated ratio (~1.0099) is
genuinely close to the 1% line regardless of its rank against stale rows, the instrument is correct,
and "the next instruction added to the send path crosses it on this axis" is a defensible forward
warning either way. Only the ranking claim needs correcting.

## Comment claims in `method_access_private_attribute.rex` -- all checked, none false

1. "`install_attribute` records the scope once per generated name" -- true, read from the source
   (the `for name in names` loop calls `record_access_scope` once per iteration).
2. "the setter's dictionary key is the attribute's name with `=` appended" -- true, read from the
   source.
3. "`CONDITION('E')` being 2 rather than 1 ... separates a real access check from a build that
   dropped the accessor" -- true, verified by mutation (see F2 above): dropping the accessor gives
   `E=1`, the real check gives `E=2`.
4. "A bodyless private attribute read from a caller the check ALLOWS ... the oracle answers the
   uninitialised instance variable and this crate refuses ... a separate gap" -- true, verified on
   the oracle: rc 0, stdout `A`, against this crate's rc 120 loud refusal.

## False prose claims found in Fix round 1 (the complete list)

* **F2's closing sentence**, "the mutation reddens exactly the two instruments added for it and
  nothing else in the workspace" -- false; a third instrument
  (`collect_stress.rs::the_l0_subset_passes_again_under_collect_on_every_allocation`) also reddens
  under the same mutation.
* **F4's ranking claim**, "which is the second-closest cell in the file to 1% after `strings`" --
  false; at least three axis/arm cells from tasks 9 and 11 (`strings`/`tw`, `alloc4c`/`tw` twice,
  `strings`/`ir`) are wider than `dispatchclass` in the committed TSV.

Everything else audited in this round -- F1's restatement, F3's contribution table, F5's assertion
account, F6's doc-comment edit, the corpus program's four factual claims, and the gates rerun at
`df799fee0` (`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --release --workspace`, `REXX_CORPUS_GATE=1 cargo test --release --workspace` at 160/160,
`REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast`) -- checked out exactly as
stated, all exit 0.

Tree left as found: no commits, no net edits, working tree clean, release binary rebuilt against
HEAD.
