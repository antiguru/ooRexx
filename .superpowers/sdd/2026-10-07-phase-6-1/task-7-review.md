# Phase 6.1 Task 7 review (6860d4f80..4de3699d9)

Probes and mutation copies under `/tmp/claude-1000/p61/t7rev/` (`m1`-`m9`, `p1`-`p9`). Every crate run
used `memcap 2G` on `rust/target/debug/rexx-run` (built 11:58, after `3b9a80364`; the binary holds
the new `an entry the library bootstrap fills` text and not `an entry owed by Phase 5`). Every oracle
run used the standard wrapper from a fresh directory. No `oracle-crashes.txt` program was run. The
mutation runs compiled `refusal_dispositions.rs` and `closed_phases.rs` on their own with `rustc
--edition 2024 --test`, with `CARGO_MANIFEST_DIR` set to a `git archive 4de3699d9` copy. Both tests
use only `std`, so the cargo build was not needed. The unmutated copy passes: 3 of 3 and 8 of 8.

### Spec Compliance

- ✅ Spec compliant for Steps 1-4. Every file the brief lists has its hunk. The test fails in each
  direction the brief names. `CLOSED` gains Phase 5 and the debt paragraph is gone. The scan covers
  `src/` and the four owner tables. NC-h and NC-i are closed. `refusal-sites.tsv` is re-derived.
  The bench comments are fixed.
- ✅ Exit criterion 1 (R8 scope) has no misses. Census (b): `object_position`,
  `method_from_source`, `method_body`, `required_source` and `use_local_in_a_method` are deleted
  (absent from my grep below). `receiver_class`, `operator_operand`, `object_method`,
  `library_source`, `setup_method` and `expose_receiver` have owner `None` and a row.
  `environment_symbol` (`.STREAM`, `redirect.rs:652`, `run.rs:3988`), `environment_entry` and
  `unreadable_collection` take `Option` and have a row. The `instruction` OPTIONS site now runs
  (probe `p9/o.rex`: `ok`, rc 0, on both engines). Census (c): `accessor_variable`,
  `delegate_variable` and `builtin_option_object` are deleted. `Refused::Raised` is
  `conversion_raised` (GUARD). The DO/LOOP and USE ARG sites agree with the oracle (`p9/d.rex`,
  `p9/u.rex`). The top-level parse failure is not a `Loud` constructor; `p9/p.rex` gets rc 221 on
  both engines, with the `&1` insert that R3 licenses.
- ⚠️ The brief cites `owners.rs:154`, `:213` and `:324-331`. At `6860d4f80` those lines are the
  `Call` split, the `EndStyle` tags and `EXCLUSION_PHASES`, and none of them names Phase 5. The only
  Phase 5 owner data was `SPLIT_TABLE_PHASES` and the count at `:468-473`, and both are handled.
  `git grep '"Phase 5"' -- rust/crates` shows only `CLOSED`, as reported. The controller can treat
  those line numbers as stale.

### Strengths

- The test's enumeration matches an independent listing exactly. I grepped `-> Loud` and `\bLoud\s*\{`
  over non-test `src/` and found 43 constructors (42 in `lib.rs` and `dispatch/native.rs:735`) and
  3 struct literals outside them (`directives.rs:64`, `dispatch/library.rs:837`, `:843`). A dump
  of the test's own `owners()` and `literals()` gives the same 43 and the same 3. Its 36 ownerless
  constructors are exactly the table's 36 rows (`diff` empty). The 7 owned constructors are
  `deferred_send`, `internal_routine`, `redirection` and `unresolved_call` (Given), and
  `named_semaphore`, `native_method` and `package_option_write` (literal phase). Every Given call
  site passes a literal or a non-`None` `&'static str` (`redirect.rs:426`, `:521`,
  `dispatch.rs:2678`, `run/call.rs:245`, `:576`).
- Each claimed failure direction bites on a scratch copy:
  - `m1`, an added `fn mutant_unlisted(what) -> Loud` with owner `None`: `mutant_unlisted
    (...lib.rs:834): no owner and no disposition row`.
  - `m2`, a row for a constructor that does not exist: `ghost_constructor: a row with no
    constructor in src/`.
  - `m3`, `native_method REHOME Phase 10`: `REHOME to Phase 10, but the constructor ... carries
    Phase("Phase 9")`.
  - `m8`, `receiver_class REHOME Phase 9`: `... carries None`.
  - `m4`, the owner-`None` call-site direction, with the `environment_entry` row deleted. Its
    `.STREAM`/owed call sites pass `None`. Result: `environment_entry (...lib.rs:556): no owner and
    no disposition row`. The test reads any `Option` owner parameter as ownerless, so it covers
    every possible call site without enumerating them. That is stricter than the brief's
    call-site reading, and correct.
- `the_check_finds_each_kind_of_disagreement` asserts the exact problem count over a fabricated
  source, so the negative cases are pinned and an extra one cannot slip in.
- `closed_phases` bites in all three places (`m9`). `unblocked_by: "Phase 5"` in `assertions.rs`
  gives `tests/assertions.rs:199: Phase 5`. `Some("Phase 5")` at `redirect.rs:652` gives
  `src/redirect.rs:652: Phase 5`. The appended exclusions row `OWNER: Phase 5 for the rest.
  EXTERNAL was DELIVERED by Phase 9.` stays open (NC-h).
- GUARD routes are cited and hold. I probed each cited route and at least one more. Programs that
  reached a GUARD: none.
  - `conversion_raised`: `RxCalcPower(o, 2)` with a raising `STRING` gives `syntax 93 93.900` on
    both engines (`p7/g1.rex`). The library-method route `RegularExpression~match(o)` gives 88.909
    on both (`p7/g9.rex`).
  - `environment_*` and `unreadable_collection`: walks of `.environment` and `.local` items,
    indexes and suppliers reach only the Phase 10 `STDQUE` refusal (`p7/g4.rex`, `g4b.rex`). A
    stream as the first clause and an `ADDRESS ... WITH OUTPUT STREAM` agree (`g2`, `g3`).
  - `operator_operand`: `.context + 1`, `\ .local` and `.array || 'x'` agree (`g5`).
  - `setup_method`: `.Class~inheritInstanceMethods(.object)` and `.class~method('DEFINECLASSMETHOD')`
    give 97.1 on both (`g6`).
  - `numeric_form_without_expression`: `interpret 'numeric form value'` gives 35.917 on both (`g7`).
  - b11: `do x over .environment` and `.methods` agree (`p9/b11.rex`).
  - `unknown_receiver`: I sent 20 messages to a WeakReference and passed one as an argument in 15
    ways (`p5/w*.rex`, `p5/a*.rex`), including `copy`, `objectName=`, `run`, `setMethod`,
    `define`, `inherit`, `enhanced`, `isA` and `Message~new`. I also tried a WeakReference
    subclass's private `setMethod`, `run`, `copy` and `OBJECT` scope (`p5/ws.rex`). Ten Method-object
    sources went through `define`, `setMethod`, `enhanced`, `defineMethods` and `run` (`p6/d.rex`):
    `.context~executable` from instance and class methods, `newFile`, native rows, `.methods`, an
    `instanceMethods` item, and a `setPrivate`d copy. A method context whose scope deleted,
    redefined or unset it was also tried (`p3/a,b,c,e.rex`), as was the queued
    `2026-10-02-context-executable-setmethod` program (`p2`). Every run agreed with the oracle, or
    raised the same error, and none printed `a message send to`.
- DEVIATION records check out:
  - `receiver_class` reaches its 11 rows: Array, Queue, Method SCOPE, Class ID, List, Supplier,
    Package, VariableReference, StackFrame and Routine refuse with the texts Deviation 28 and entry
    33 quote (`p4`). Table answers the Phase 9 text Deviation 28 states.
  - Entry 33's crashing rows cite scout A (`scout-a-report.md:1756-1765`, `:1813`) and Task 4
    (`phase-6-1-gate.md` "## Task 4" rows 25-27). Its non-crashing rows match scout A `:66` and
    `:1753-1764`.
  - Deviation 29's mechanism and its "three runs of three" are in
    `records/2026-08-17-phase-5a/upstream-tickets.md:235-290`, and ours refuses as stated
    (`p7/dv.rex`).
  - `stale_handle`'s quoted row is at `phase-4-exclusions.txt:6082`. `chunk_refused`'s cited plan
    has `ChunkTooLarge` at `:76-77`.
- Concern 4: Phase 7 is CLOSED 2026-09-13 (`2026-07-27-rust-rewrite.md:659`). Two checks now
  reject a Phase 7 tag in any of the seven tag tables, so the deleted zero counts were redundant
  and no policing is lost. The first is `assert_owner_strings_are_split_table_phases`
  (`owners.rs:348-368`), now that `SPLIT_TABLE_PHASES` is `["4b", "4c"]`. The second is
  `no_owner_table_names_a_closed_phase`, which reads `owners.rs`'s literals (`m9` shows it bites).
  The exempt data files `bif-exempt.txt` and `keyword-exempt.txt` are not scanned. Their
  `unblocked_by` columns are held by the pruned `PHASES` vocabularies (`bif_assertions.rs:781`,
  `keyword_assertions.rs:668`), so that gap is closed too.
- Concern 6 holds. `p8/a.rex` puts `runDynamicSource` in a test-case class with `hex` and `bin`:
  the oracle prints `AB A`, rc 0, and ours gives `method "NEW" of class "Routine" is not implemented
  (Phase 9)`, rc 120. `p8/b.rex` (`Routine~new` with a package) refuses the same way, and
  `p8/c.rex` (the 2-argument form) answers `AB` on both. The harness gap is real: `p8/d.rex` is the
  harness shape, and both engines give `97.1 Object "SELF" does not understand message "HEX"`.
- Sizes: `receiver_class` has 41 sites and `unknown_receiver` 27, from a grep of non-test `src/`.
  This matches the report.

### Issues

#### Critical (Must Fix)

None.

#### Important (Should Fix)

1. **The disposition test passes three ownerless refusals with no row**
   (`rust/crates/rexx-exec/tests/refusal_dispositions.rs`). These lines refer to the review diff.
   Each mutation below leaves `every_ownerless_refusal_has_a_disposition` green.
   - `m5`: `fn mutant_self(what: &str) -> Self { Self { message: owned_message(what, None) } }`
     inside `impl Loud`. `constructors()` (diff `:2899-2911`) takes only a `Loud` return type, and
     `literals()` (`:3057`) looks only for `Loud {`. A constructor written in the idiomatic `-> Self`
     form is therefore not found at all.
   - `m6`: a constructor whose early-return branch is `owned_message(what, Some("Phase 9"))` and
     whose tail is `message: what.to_string()`. `owner_in` (`:2992`) reads only the first
     `owned_message(` and answers Phase 9.
   - `m7`: `Refused::Mutant => return Loud { message: String::new() }.into(),` placed in
     `dispatch/library.rs` just above the `ClassicStyle` arm. The literal's 12-line text window
     (`:3081`) takes the next literal's `Some("Phase 10")`.

   Today's tree has none of these shapes, so the current verdict is right; see Strengths for the
   independent grep. But this file is exit criterion 1's instrument, and its own doc
   (`:2699-2715`) claims it reads every constructor and every literal. Fix:
   - inside `impl Loud`, accept `-> Self` and `Self {`;
   - read every `message:` field of a body, and count any field that is not an `owned_message`
     call with a non-`None` owner as ownerless;
   - bound a literal's text by brace matching (`arguments` already does the depth walk) instead of
     a 12-line window;
   - add the three mutations to `the_check_finds_each_kind_of_disagreement`.

#### Minor (Nice to Have)

1. `rust/corpus/refusal-sites.tsv`, the `Loud receiver_class` row (diff `:748`), still records
   `diverges yes` with the witness `a. = 'dflt' then a.~length`. That program prints `4`, rc 0, on
   both engines (`p1/s.rex`), so the row is false. It also contradicts the new DEVIATION
   disposition. The row was stale at base too, but this task re-derived the table and wrote hand
   columns for three other constructors. Fix: use a wrong-type row as the witness, for example
   `.t~new~go(.array~method('ITEMS'))`, which is rc 120 on ours.
2. `dispatch/context.rs:457`: `Loud::receiver_class("a package table this crate did not build")`
   stays a DEVIATION site. It sits on the routine `.context~executable` path beside its
   `unknown_receiver` siblings (diff `:1020-1028`), and no borrowed native row reaches it. It is a
   guard that is filed under the DEVIATION constructor. Move it to `unknown_receiver`.
3. `tests/closed_phases.rs`, the `open_owners` doc (diff `:2408-2412`), says "the phase is read from
   the owner sentence up to its resolution". The code does not do that. When any resolution is
   found, the owner is skipped (`resolution.is_some()` then `continue`, `:2471`), and the
   `.filter_map(...).min()` position (`:2462-2470`) is never read. Delete the sentence, and reduce
   the `.min()` to `any`.
4. Wrong-type borrowed rows still answer `native_method` `(Phase 9)` on Directory, StringTable,
   Stem, Set, Bag and Relation ITEMS, MutableBuffer LENGTH and Message SEND (`p4`). Deviation 28
   records only Table. A Phase 9 label on a read that Deviation 28 calls not a specified observable
   tells a reader to wait for Phase 9. This is class (a), outside R8, so it belongs in a queued item
   or in Task 12's Phase 9 amendment.
5. Deviation 28 (`phase-4-exclusions.txt`, diff `:262-283`) names only `self~run(m)`. The same
   read is reached through `enhanced`. In `p6/d.rex` (first version), `.object~enhanced(d)` with
   `d['E9'] = .array~method('ITEMS')` and then `e~send('E9')` gives `receiver_class`, rc 120, on
   ours and rc 0 on the oracle. Widen the wording to "installed or run on".
6. The `Literals` rows' `unblocked_by: "Phase 9"` (`tests/assertions.rs:199-277`) names only the
   interpreter half. Phase 9 alone will not make these rows pass in this harness, because the
   harness supplies no `self` (Concern 2, confirmed by `p8/d.rex`). The doc comment says so, but
   the field's meaning ("the sub-phase that would actually unblock it") does not hold. Queue the
   harness half.
7. The perf judgement (Concern 5): the refusal relabels are cold, since they are constructor swaps
   on error branches. `DirectoryEntry::Owed` and `owed_entry_owner` widen from `Option<&str>` (16
   bytes) to `Option<Option<&str>>` (24 bytes, measured with `size_of`). That value is returned on
   every store-backed directory read (`dispatch/hash.rs:1684`), a warm path for `.NAME` and
   `Directory` reads. This is likely below noise, but nothing measured it, so Task 9's cumulative
   run should include a Directory-read program. The type is sound: the outer `None` means "not an
   owed item" and `Some(None)` means "owed, no owner". The compiler enforces this at every
   site (`environment.rs:530`, `route.rs:34`, `identities.rs:667`, `hash.rs:435`, `:1684`,
   `class_protocol.rs:514`, `:805`, `:1110`).

Incidental, not this task: an enhanced object's name in a 97.2 message differs between the engines.
The program is `e = .object~enhanced(.methods)` / `e~p` / `::method p private` (`p6/en.rex`). Ours
says `Object "an Object" cannot accept private message "P"` and the oracle says `Object "enhanced
Object" ...`. A grep of the exclusions file, `queued/` and `corpus/` for `enhanced Object` finds no
record of it.

### Assessment

**Task quality:** Needs fixes

**Reasoning:** The dispositions, records, `closed_phases` widening and NC-h/NC-i closure are correct
and verified by probes and mutations. Exit criterion 1 is met today, and its test's enumeration
equals an independent grep. The test is blind to three ownerless-refusal shapes, though: a `-> Self`
constructor, a mixed-branch body, and a literal next to an owned one. Exit criterion 1 rests on
this test, so it should be closed before the task is accepted.

## Fix round 1

Checked at `b71606f16`. The diff reviewed is `git diff 4de3699d9..b71606f16`. Crate source changed
only in `732c65404` and `58a558ff1`; `git diff --stat 58a558ff1 b71606f16 -- rust/crates` is empty.
Mutations ran on a fresh `git archive b71606f16` copy, with the tests compiled standalone as before
(`/tmp/claude-1000/p61/t7rev/f1/`).

**Verdict: Approved.** The Important finding is fixed and every Minor is addressed. Three new
Minor findings are listed below; none of them blocks the task.

### Important 1: fixed

All nine original mutations, reapplied to the new tree, fail the test:

| mutation | failure |
|---|---|
| `m1` | `mutant_unlisted (crates/rexx-exec/src/lib.rs:836): no owner and no disposition row` |
| `m2` | `ghost_constructor: a row with no constructor in src/` |
| `m3` | `native_method: REHOME to Phase 10, but the constructor ... carries Phase("Phase 9")` |
| `m4` | `environment_entry (...lib.rs:558): no owner and no disposition row` |
| `m5` (was green) | `mutant_self (...lib.rs:836): no owner and no disposition row` |
| `m6` (was green) | `mutant_branch (...lib.rs:836): no owner and no disposition row` |
| `m7` (was green) | `crates/rexx-exec/src/dispatch/library.rs:836: a Loud struct literal with no owner` |
| `m8` | `receiver_class: REHOME to Phase 9, but the constructor ... carries None` |
| `m9` | `closed_phases` fails all three checks: `tests/assertions.rs:200`, `src/redirect.rs:652` and the NC-h row `OWNER: Phase 5 for the rest` |

The unmutated copy passes `refusal_dispositions` 3/3 and `closed_phases` 8/8. A dump of the new
scanner's own enumeration still matches the independent grep: 43 constructors, and the ownerless
set equals the table's rows (`diff` empty). The 3 literals outside a constructor are the same three,
read with the same owners.

Two more mutations aimed at the new code also fail as they should:
- `m11`: an `owned_message(.., Some("Phase 9"))` bound to a `let`, followed by an ownerless
  `message: what.to_string()`, gives `mutant_via_let ... no owner`.
- `m12`: an ownerless `Self { .. }` literal in a non-constructor helper inside `impl Loud` gives
  `lib.rs:837: a Loud struct literal with no owner`.

### Minors 1-7: each addressed

1. `refusal-sites.tsv:275`: the `receiver_class` witness is now the `Array ITEMS` wrong-type row,
   which is rc 120 on ours (re-probed).
2. `context.rs:457` now calls `unknown_receiver`. Grep counts are 40 `receiver_class` sites and 28
   `unknown_receiver` sites, matching the report.
3. The false doc sentence in `open_owners` is deleted, and the `.min()` is now `any`.
4. Re-probed with the debug binary built at 12:50:57, just before `58a558ff1`: Directory,
   StringTable, Stem, Set, Bag, Relation and Table ITEMS, MutableBuffer LENGTH and Message SEND each
   answer `(Phase 9)` at rc 120 (`/tmp/claude-1000/p61/t7rev/f1p`). Array ITEMS answers
   `receiver_class`. The queued item `2026-10-09-wrong-type-native-rows-name-phase-9` is true
   except for one attribution; see New Minor 2.
5. Deviation 28 now says "installed on or run on" and records the `enhanced` route with its probe.
6. The queued item `2026-10-09-literals-rows-need-a-test-case` is true: `p8/d.rex` and the
   implementer's `lit3` give 97.1 on both engines. The `EXEMPT` doc comment points at the item.
7. Perf. I reran `callgrind.sh -r 1 -p "dirread rexxcps"` on the implementer's binaries. I verified
   that `src-head2` is byte-identical to `58a558ff1`'s `rust/crates` (`diff -rq` empty), and that
   `src-base` still carries `Some("Phase 5")`. I did not rebuild the binaries, so the check that
   each binary came from its source tree rests on the implementer's `Compiling` lines.

   | program | base | fix | delta |
   |---|---:|---:|---:|
   | dirread | 2647797400 Ir | 2645795220 Ir | -0.0756% |
   | rexxcps | | | -0.0000% |

   Both deltas equal the report's figures. `#[cold]` on two refusal constructors is a sound fix
   for the inlining the report diagnosed in `array_slots`.

### New Minor findings

1. **`m10` stays green.** An `impl Loud` block indented inside a nested module, holding
   `pub(crate) fn mutant_nested(what) -> Self { Self { message: crate::owned_message(what, None) } }`,
   is not seen. The cause is `in_impl_loud`, which only matches `impl Loud` at column 0 and ends
   the block at a column-0 `}` (`refusal_dispositions.rs`, fix diff `:187-201`). The code is legal
   Rust, since a child module can name the private `Loud`, but no such block exists today. Fix:
   track the `impl Loud` line's indent and close at the matching indented `}`.
2. **One wrong attribution in the wrong-type queued item.** The item puts every ITEMS row under
   `dispatch/hash.rs` `not_this_task`. Stem ITEMS actually refuses through `stem_refusal`
   (`dispatch/hash/stem.rs:528-529`), and Relation's site is `dispatch/hash/relation.rs:123-124`.
   Name the sites per row.
3. **Set sizes in prose.** The queued `2026-10-09-literals-rows-need-a-test-case.md` says "twelve
   `Literals` EXEMPT rows", and the report says the same. The global prose rule bans stating a
   set's size, so use "the `Literals` EXEMPT rows".

Counts after the fix round: 0 Critical, 0 Important, 3 Minor (new).
