# Phase 5c follow-up — `MutableBuffer` gets state

Spec: `docs/superpowers/specs/2026-09-04-phase-5e-mutablebuffer.md`. Read it first; this plan does
not restate its measurements.

**This is 5c's unfinished business, not a new phase.** Moritz ruled 2026-09-04, answering the
spec's D81 and the plan review question that asked whether `5e` should exist: `MutableBuffer`'s rows
are already owned by `5c` in `class-set.txt`'s `method-owner` column, the work is 5c's, and a new id
would say this is new scope when it is not.

**"Re-open" is an ownership word here and not a `CLOSED_PHASES` change**, which is the reading a
reviewer should check hardest. Measured: removing `"5c"` from `CLOSED_PHASES` would redden
`every_closed_phase_this_table_owns_rows_for_is_gated`, which requires a phase with a committed
`corpus/phase-5c.txt` to be in it. And nothing needs it to be removed —

* the method-body gate is a **drift** gate, and `loud` → `answers` is explicitly ungated progress;
* all 51 of `MutableBuffer`'s gate table C rows are `hasMethod` readbacks, which already agree and
  which implementing a body cannot move.

So **D81 dissolves and this plan has no ownership task.** If a reviewer finds a reason 5c must
actually leave `CLOSED_PHASES`, that is a finding worth more than the rest of this review.

**The hollowness check is already landed** at `0ef9d5064`, ahead of this work and deliberately so —
the spec's risk section said to land it first, because both it and this phase move rows in
`corpus/method-bodies.txt` and one diff carrying both would stop being readable. The 71
`unanswered` rows are already in the baseline, so any row this phase moves is this phase's.

BASE for Task 1 is the commit this plan lands in.

---

## Global constraints

Every task, no exceptions.

* **Commit before the full gate run, not after**, per `rust/CLAUDE.md`'s Gates section: fast checks
  yourself, commit code and report together with the report's gate cells left as `**G1**`–`**G7**`,
  start the suite in the background with the commit sha as the status file's first line and a
  pidfile beside it, report `committed at <sha>, gates running, statuses at <path>` and **stop**.
  The tree belongs to the gate run until its status file says `finished`.
* **Every claim gets a red control**, predicted before it is run and marked confirmed, falsified or
  unobservable. A control that varies something *about* the change proves nothing.
* **Both engines every time**: `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker` — that exact spelling.
* Three separate descriptors, never `2>&1`, `$?` captured immediately.
* Oracle runs from a fresh empty directory, wrapped as
  `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`.
* **Never `git checkout -- <path>`** on a file you edited; `cp` to your scratch directory and
  restore from the copy, then `touch` it — `cp -a` preserves mtime and cargo will keep the stale
  binary.
* No `unsafe`; the workspace lint is `deny` and it is Moritz's call per site.
* Never `git add -A`; never amend; `git commit -F <file>` with paths named; confirm `Cargo.lock` is
  not staged unless the task deliberately changed it.
* **A method that exists and does nothing is not implemented** — `rust/CLAUDE.md`, and this phase
  exists because of it. Do not add a name that answers nothing to close a row.
* **The corpus control that can go red is `REXX_CORPUS_GATE=1`.** The plain `--test corpus` binary
  is report mode — `corpus_differential` asserts `!gate || mismatches.is_empty()` — so it exits 0 on
  a divergence and prints `N of M matching` to stderr. Measured by Task 3a: M1 exited 0 at `358 of
  359`. Cite the STRICT run and the matching line, never the plain exit.
* **Mutation runs build with `--profile mutation`** (release plus thin LTO, own `target/mutation/`),
  so the release binary the witnesses run on is never a mutant's.

---

## Task 0 — give the instrument teeth, before anything is implemented

**Lands alone, ahead of every other task, and changes no behaviour.**

`method_bodies.rs`'s `RECEIVER_OVERRIDES` (`:370`) is the sanctioned lever — one entry today, for
`DateTime`, and `check_receivers_match_table_c` already exempts the classes in it. Add
`MutableBuffer` with the receiver `.MutableBuffer~new('abc')`.

**Why it must come first.** Measured against a rebuilt pristine `c214add97` binary, all 51 probes
under the new receiver are `LOUD=51, OTHER=0` — **the change moves nothing in the committed table
today**, exactly as the hollowness check at `0ef9d5064` moved nothing when it landed. So the
instrument can be sharpened while there is still nothing for it to catch, and every later row
movement is measured against the sharper instrument rather than the blunt one.

**What it buys, measured**: ten rows become unfakeable — `length` 3 against 0, `string`/`makeString`/
`space`/`subWords`/`translate`/`upper`/`lower` content against empty, `words` 1 against 0,
`makeArray` non-empty. **41 of 51 remain fakeable**, which is why Task 2 also carries a corpus
witness.

**`delete` and `delStr` are not among the ten and that is not a bug**: sent with no arguments they
delete everything, so they answer empty on `'abc'` as on an empty buffer.

---

## Task 1 — where the bytes live, decided by measurement

**Answers D80, and builds nothing that ships.**

The spec offers three shapes: an object variable in `Body::Instance`'s `pools`; a new boxed `Body`
variant carrying `Bytes` plus a capacity, on `Body::Native`'s precedent; or `NativeObject`'s
`entries` map. **`rexx-core/src/body.rs` has a width assertion and it is the constraint on all
three** — read it before proposing anything.

**First, the question nobody has asked**: where the byte machinery already is. Task 0 measured
that it is **not** in `String`'s method surface — `NATIVE_METHODS` binds `LENGTH REVERSE SIGN
UPPER NEW` for `String` and `'abc'~substr(2)` is rc 120 on both engines. It is in the builtin
*functions*, `rexx-exec/src/builtin/string.rs` and `word.rs`, with `(interp, name, Args)`
signatures. Find out whether those factor over a `&[u8]` a buffer can hand them; if they do, say so
and the phase gets smaller.

**Measure, do not choose on taste.** Build the two live candidates far enough to run an append loop
and a length loop, and report instructions for each with the sha256 of each binary. The prototype
lives in a scratch crate under `claude-build-scratch/`, kept until the phase closes and its figures
reproducible from there — not committed, and not thrown away.

**`defaultSize` is observable and the representation carries it** — Task 0 settled the question
this paragraph used to leave open. The C++ carries `bufferLength` *and* `defaultSize`;
`getBufferSize` reports the former, and `setBufferSize(0)` is the one path that reads the latter
(`MutableBufferClass.cpp:686`-`:691`): it shrinks capacity back to `defaultSize`, which is the
constructor's second argument or 256. Oracle: `new(copies('x',400))` then `setBufferSize(0)` reads
`0 256`; with a second argument of `300`, `0 300`; `new('abc',500)` grown to `1203 2000` then
`setBufferSize(0)`, `0 500`. `delete` and `setText('')` leave capacity alone. Growth is
`max(needed, 2 * capacity)` (`ensureCapacity`, `:243`): `new('',10)` under seven-byte appends reads
`10 20 40 40 40 80 80 80`.

---

## Task 2 — the constructor carries state, and the witness proves it

**The representation is decided** (Task 1, `records/2026-09-04-phase-5c-followup/task-1-report.md`
§4): `Body::Instance` gains `native: Option<Box<BufferState>>` with `BufferState { bytes: Vec<u8>,
capacity: usize, default_size: usize }`; `size_of::<Body>()` stays 80. Rendering per §4.2 — the
infallible renderings answer the bytes, `MAKESTRING` stays unbound until Task 3. Two more witnesses
come from Task 1's measurements: `trace i` on `x = buf` shows the contents, and a `MutableBuffer`
subclass with `EXPOSE` runs.

Implement that representation and the methods the witness needs: `~new` keeping what it is
given, `~length`, `~string`, `~endsWith`, `~append`, `~delstr`, `~getBufferSize`, `~setBufferSize`.

**Capacity is a real observable and has three behaviours to match**, all measured: it defaults to
256; the constructor's second argument sets it; `new(copies('x',400))` reports `400`, so the initial
string raises it; and **`setBufferSize` below the contents truncates them** — `new('abcdef',100)`
then `setBufferSize(3)` gives `len 3 cap 3 string abc`.

`dispatch.rs`'s constructor doc says the contents "are not kept, and nothing that would read them
answers", and `a_constructor_taking_arguments_answers_an_instance_and_refuses_its_state` asserts
that pair. **Both become false and both are yours to correct** — the test becomes the assertion that
state *is* kept.

**The witness is `corpus/lang/mutablebuffer_state.rex`**, verified reachable inside this phase alone
— no `String` method, no `File`, no native entry point, and `copies('x',400)` answers on both
engines. Oracle rc 0, stderr empty, nine lines: `6 / abcdef / 1 / 9 / abcdefghi / abcdefgh / 999 /
400 / "8 500"`. Against the stateless stub it is **rc 168, diverging at line 2** — `0` against `6` —
so a stub that reads its arguments still cannot pass it. Add the truncation case. **File it in
the same commit** — `corpus/phase-5c.txt` and `coverage.rs`'s `EXPECTED_SUBSET_5C` together, verified
in a `git archive` extract — because the differential runs only `SUBSET_FILES` programs and an entry
in `corpus/unfiled.txt` is a witness nothing runs. (Amended after Task 2: it landed unfiled and the
controller filed it; every later witness is filed by the task that writes it.) Every `corpus/lang`
program also needs its `rexx-parse/tests/sourceline_oracle/<name>.txt` companion, generated by the
driver in that test's module comment.

**Keep `makeString` out of it until Task 3.** `say buf` routes through `makeString`, measured, so
including it makes the program unpassable until that lands; until then `say buf` stays loud rather
than wrong.

**Do not name the buffer variable `b`.** A symbol `b` followed by a quoted string parses as a binary
literal — `say 'd' b~length '['b~string']'` is error 15.4.

---

## Task 3 — the rest of the instance surface

The remaining rows: the caseless family, the word family, the search family, the mutators.
**Each family is its own commit with its own gate run**, and **each needs its own corpus witness** —
Task 2's program exercises eight names and says nothing about the other 45. A family that lands
without a witness has landed behind an instrument its own stub could satisfy.

**Several, decided by Task 1 (§5)**: first a commit that extracts the byte cores out of
`builtin/string.rs` and `word.rs` into plain functions over `&[u8]` (the builtin tests are its
witness; Task 2 does `delstr`'s as the first instance), then four commits — **readers over cores**
plus the derived readers (`substr [] pos lastPos countStr verify subWord word wordIndex wordLength
words wordPos contains containsWord startsWith match matchChar subChar`), **mutators over cores**
(`insert overlay replaceAt []= changeStr upper lower translate space delWord delete`; **this list
omitted `setText`, corrected 2026-09-05 after Task 3e, which bound it as the last unbound instance
row -- it belonged here**),
**caseless** (a comparator through the cores; its witness holds mixed-case data), and
**conversion** (`makeString string makeArray subWords`, the commit that flips `say buf` and carries
its own witness). Each commit its own gate run and its own corpus witness, **filed in that commit** (see Task 2).
**The cores landed at `641e76b84`** (Task 3a): `substr_bytes insert_bytes overlay_bytes space_bytes
changestr_bytes translate_bytes verify_bytes case_shift_bytes` in `builtin/string.rs`;
`word_count word_range subword_range delword_bytes wordpos_bytes word_slices` in `builtin/word.rs`;
a byte `start` is 0-based, a word `position` 1-based, an omitted length `Option<usize>`.
`MutableBuffer~verify` answers `counted` on every path and is the only caller that reaches
`verify_bytes`'s start-past-the-end guard; whether the builtin follows is open
(`task-3a-report.md` §6.1).
The conversion commit carries one more constraint, measured after Task 2: `buf == 'abc'` is Object
identity on the oracle (`0`) while `'abc' == buf` and `length(buf)` request the buffer's string
(`1`, `3`) — binding `MAKESTRING` must leave the receiver-side comparison an identity compare.
Whenever a commit touches `src/*.rs`, its fast checks include `--test refusal_sites`:
`corpus/refusal-sites.tsv` cites constructor definitions by line and a one-line insertion above
them reddens G3–G5, as Task 2's did.

**The oracle's own `MutableBufferClass.cpp` is the authority**, and `utilityclasses.xml` documents
the surface. Where they disagree, the oracle wins and the divergence is recorded. Note that `delete`
and `delStr` are **one method under two documented names** (both cite `mthMutableBufferDelStr`), so
any row arithmetic must say whether it counts names or methods; the table counts names.

---

## Task 4 — file the witnesses, and close

**No `CLOSED_PHASES` change, no new subset file, no `SUBSET_FILES` row** — see the head of this
plan. The witnesses are already filed, each by the task that wrote it (amended after Task 2); this
task confirms `corpus/unfiled.txt` carries none of them and that `EXPECTED_SUBSET_5C` and
`phase-5c.txt` agree line for line.

**Verify by running it in a `git archive` extract with its own `CARGO_TARGET_DIR`** before
committing — 5c's own flip did not work and that is how it was found. **Enumerate from the tree,
not from this plan.**

---

## The gate

**The witness passes, and the strengthened rows move.** Specifically: `mutablebuffer_state.rex` and
Task 3's family witnesses agree with the oracle byte for byte on all three descriptors, both
engines; and no `MutableBuffer` row moves to `diverge`.

**"Every one of the 51 rows moves `loud` → `answers`" is NOT the gate, and the spec's version of it
is retired.** The plan review built the stub that satisfies it: 51 rows bound to seven functions,
about 90 lines, `native_mutable_buffer_new` untouched and still discarding its argument, and all 51
agree on all three descriptors on both engines. The oracle's 38 argument-error rows are nearly
information-free — 32 of them are one message character for character — and both raisers already
exist in this crate and are already byte-identical. **A criterion a stub satisfies is not a
criterion**, and this is the third time the same idea has failed here: `hasMethod` readbacks, then
"the row answers", then "the row answers a zero-argument send".

**`File` is not in the gate and not in this phase.** `.File~new('/tmp')~name` needs three owners —
`MutableBuffer`'s methods (this phase), `file_qualify` (Phase 7, `deferred(..., Family::File)`), and
`String~lastPos`/`~substr` (5c's own, inside String's 112 loud rows).

**And `File`'s 50 `unanswered` rows will not move at all**, which an earlier draft of this plan got
wrong. `constructs` is `class.construction.is_some()` — a committed `class-set.txt` field, not a
runtime fact — and `File`'s is `-`; its probe receiver is a bare `.File~new`, which raises 93.901 at
rc 163 on the oracle and on both engines. Those rows stay `unanswered` until someone gives `File` a
construction expression, whatever gets implemented.

---

## Notes carried from the plan review, none of them findings

* **G6 and G7 gate nothing G4 does not.** `verdict_is_gated` is `corpus_gate() && (closing_phase()
  == Some(phase) || CLOSED_PHASES.contains(&phase))`, and both `5c` and `5d` are already in
  `CLOSED_PHASES` — so the two phase-gate commands re-gate what `REXX_CORPUS_GATE=1` already gated,
  over a subset of the binaries. Not wrong; they read as extra assurance and are not. They start
  doing work again the first time a phase is gated *before* it closes.
* **`RECEIVER_OVERRIDES` is a general instrument nobody has swept.** The same question — does the
  documented receiver make the row unfalsifiable — applies to every `covered` class whose bare
  `~new` builds something empty: `String`, `Queue`, `Array`, `Stem`, `List`, `CircularQueue`.
  Task 0 is one loop away from knowing how wide this is; taking that loop is optional and reporting
  the number is not.
* **`~name` is cwd-independent, `~absolutePath` is not.** Any future `File` witness prints names,
  never qualified paths.
* **No engine divergence anywhere in the review** — 102 probe runs, the stub, the witness, the
  arity and `File` probes, all identical between `ir` and `tree-walker`.

---

## Open for a second review, if one is taken

1. ~~**Is Task 3 one task or several?**~~ Several; decided by Task 1, see Task 3.
2. **Should Task 0 sweep the other `covered` classes** rather than only fixing `MutableBuffer`'s
   receiver? It is the same one-line mechanism per class and nobody knows how many rows it would
   sharpen.
3. ~~**Does `defaultSize` need to exist in the representation at all?**~~ Yes; `setBufferSize(0)`
   reads it. Measured by Task 0, carried by Task 1's recommendation.
