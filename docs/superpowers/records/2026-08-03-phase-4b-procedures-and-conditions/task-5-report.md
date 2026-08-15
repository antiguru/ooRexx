# Task 5 report: `PROCEDURE`, `PROCEDURE EXPOSE`, `USE ARG` and `USE LOCAL`

Commit: **`8b87195bc5c510612761c75d5449f35e540c7106`** (`8b87195b`), on `plan/rust-rewrite`.
Baseline confirmed green before any change: `32a667ac`, `cargo test --workspace` 901 passed / 0 failed / 4 ignored, corpus `34 of 34 matching`, assertions `4224 of 4259`.

---

## 1. The redirect design, and the two measurements that chose it

**Choice: the indirection lives in `RootSet`**, as a `Vec<Option<usize>>` named `aliases`, exactly parallel to the `slots` arena. `Some(target)` at absolute position `p` means every read and write addressed to `p` is served by absolute position `target`. `slot`/`set_slot`/`clear_slot` each resolve through one shared private `resolve`, so they cannot come apart.

### 1a. The site count, measured myself

The brief records **26** at `344677e6` (`eval.rs` 3, `stem.rs` 12, `plan.rs` 4, `lib.rs` 2, `run.rs` 5) and warns it has gone stale once. At `32a667ac` I count **27** — it moved again, by one, in `run.rs` (Task 4's expression-call form).

Command, with doc-comment lines filtered as the brief requires:

```
grep -n "activation()\.frame\|activation_mut()\.frame\|\.frame\b" *.rs \
  | grep -v "^\S*:[0-9]*: *//" | grep -v "^\S*:[0-9]*: *///"
```

A breakdown the brief's figure does not draw, and which is the one that actually matters for costing the two routes — `#[cfg(test)]` boundaries are at `run.rs:4518`, `stem.rs:432`, `plan.rs:615`, `eval.rs:922`:

| file | production | in `mod tests` | total |
|---|---|---|---|
| `run.rs` | 5 | 1 | 6 |
| `stem.rs` | 7 | 5 | 12 |
| `plan.rs` | 1 | 3 | 4 |
| `lib.rs` | 2 | 0 | 2 |
| `eval.rs` | 0 | 3 | 3 |
| **total** | **15** | **12** | **27** |

All three `eval.rs` sites are inside its test module, which the brief's flat count does not distinguish.

### 1b. What decided it, beyond the count

The `Interp` route is described as "slot resolution returns a `(SlotFrame, usize)` pair". **It does not compose with the actual hot path.** `Interp::read` (`crates/rexx-exec/src/lib.rs:1175`) resolves through `code.slots` — the plan's `by_symbol` map — and calls `self.roots.slot(frame, slot)` directly, reaching `slot_of` only on a fallback miss. So a pair-returning `slot_of` would not cover the hot read at all; the `Interp` route would in practice mean writing the identical alias check by hand at each of the 15 production sites. Same instruction count on the hot path, fifteen places to get it wrong instead of one, and a further copy for every site a later task adds.

### 1c. The hot-path cost, measured

Two release binaries: `rexx-run` built at `32a667ac` (no check) and at this task's tree (check present). Both produce identical output on both programs. `rexx-time --warmup 3 --runs 15`, three rounds for the stem case.

| benchmark | baseline median | with redirect | delta |
|---|---|---|---|
| variable-heavy (400k iterations, 6 simple-variable reads/writes each) | 1285.166 ms | 1283.286 ms | **−0.15 %** |
| stem-heavy round 1 (240k compound accesses) | 324.784 ms | 333.993 ms | +2.8 % |
| stem-heavy round 2 | 320.223 ms | 330.713 ms | +3.3 % |
| stem-heavy round 3 | 325.507 ms | 327.836 ms | +0.7 % |

**The honest summary: no measurable cost on variable lookup, and about +2 % on stem-heavy code.** The stem figure is a small-signal measurement — round-to-round noise is of the same order as the effect within any single round — but the direction is consistent, slower in 3 of 3 rounds. I am not claiming better precision than that.

### 1d. The design's stated shape is wrong, and this is the program that shows it

The brief and D9r both say: *"The redirect is a bitset over slot indices plus one target `SlotFrame`, not a name-keyed map."* The index-keyed half is right. **The "one target `SlotFrame`" half is refuted.** One `PROCEDURE` can expose two names that resolve to two *different* frames.

Run from `/tmp/.../scratchpad/t5probe1` (freshly `mkdir`ed):

```rexx
n = 'from-a'
m = 'from-a-m'
call bee
say 'a sees:' n m
exit
bee: procedure expose n
m = 'from-bee-m'
call cee
say 'bee sees:' n m
return
cee: procedure expose n m
n = 'set-by-cee'
m = 'set-by-cee-m'
return
```

Oracle, rc 0, empty stderr:

```
bee sees: set-by-cee set-by-cee-m
a sees: set-by-cee from-a-m
```

`cee` exposes `n` and `m` in one `PROCEDURE`. `n` chases through `bee`'s own alias up to `a`'s frame; `m` is `bee`'s own local and stops at `bee`'s frame. A single target frame per callee gets exactly one of the two right, whichever frame it picked. The redirect is therefore per slot, which the flat parallel vector gives for free.

This is pinned by `one_procedure_can_expose_names_living_in_two_different_frames` and by the corpus program, and it is the case that cannot pass by accident.

---

## 2. `RootSet::grow_slots` — what I did about it

**The panic stands, unchanged in its `4a invariant` wording, and `crates/rexx-core/tests/collect.rs`'s `#[should_panic(expected = "4a invariant")]` is untouched.** No BLOCKED report needed.

What makes it hold is an ordering decision: **`PROCEDURE` allocates the callee's frame itself, rather than `CALL` allocating it in advance.** At the moment the `PROCEDURE` arm runs, the callee is still sharing the caller's frame (Task 3's default), so the caller's frame is the top one, and:

* Every exposed name is resolved through `Interp::slot_of` **then** — plan, then `extra`, then grow. Any grow is a top-frame grow.
* Only after that is the callee's own frame pushed and the aliases installed. `alias_slot` writes into the *callee's* frame and only reads the target's position, so it allocates nothing in the caller.

The residual case D9r names does occur and is covered. `ZQXW` appears in no instruction of either routine (only inside string literals), so the plan has no slot for it:

```rexx
nm = 'ZQXW'
call sub
interpret "say 'caller:' zqxw"
exit
sub: procedure expose (nm)
interpret "zqxw = 'set-in-callee'"
return
```

Oracle and this crate both print `caller: set-in-callee`. This needs the grown binding to reach the caller's `extra`, which `exec_procedure` does explicitly before it isolates; test `a_computed_expose_of_a_name_no_instruction_mentions_survives_the_return`.

**The two `rexx-core` comments predicting a relaxation are corrected**, as the brief requires:

* `grow_slots`'s own doc said "4b must revisit it before it has more than one live frame" and "4b either grows a non-top frame or binds an exposed name to a slot in the caller's frame at call time. Deciding which is 4b's." It now records that 4b decided, took the second, and why neither half needs a non-top grow. The panic *string* also said "4b must bind exposed names differently rather than relax this" — a pending obligation now discharged — so its trailing prose is updated while `4a invariant` is kept verbatim.
* `collect.rs`'s `growing_a_frame_that_is_not_the_top_one_panics` doc said "4b must find and remove this panic deliberately when it relaxes the rule". Corrected. That comment also had the callee setting `zzz = 5` and the caller printing `9`, which no run produces; noted and fixed to `5`.

---

## 3. Oracle transcripts

Every probe was run from a directory I created myself, never the scratchpad root, and wrapped as
`( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`
with stdout, stderr and exit status kept as three separate descriptors.

Directories: `t5probe1`, `t5probe2`, `t5probe3`, `t5probe4`, all under
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/`.

### 3a. Exposure is transitive (`t5probe1`)

The brief's own program, verbatim, gives the expected `bee sees after c: set-by-cee` / `a sees: set-by-cee`. The stronger two-frame version is §1d above. **How I satisfied myself it is transitive** is not that program alone: the mechanism is that `slot_ref` chases before returning, and `exec_procedure` resolves against *the frame in force*, which for `cee` is `bee`'s and already carries `bee`'s alias. Mutating `slot_ref` to skip the chase (`SlotRef(frame.start + index)`) makes both transitivity tests fail — see §6.

### 3b. The indirect form is plural, and exposes its own selector (`t5probe1`)

```rexx
list = 'ALPHA BETA'
alpha = 'a-in-caller'
beta = 'b-in-caller'
gamma = 'g-in-caller'
call sub
say alpha beta gamma
exit
sub: procedure expose (list)
alpha = 'a-set'
beta = 'b-set'
gamma = 'g-set'
return
```
→ `a-set b-set g-in-caller`, rc 0. `GAMMA` is the control: it is never named, and an implementation that exposed everything would pass the first two words and fail on it.

```rexx
v = 'zzz'
zzz = 'z-in-caller'
call sub
say 'caller v:' v 'caller zzz:' zzz
exit
sub: procedure expose (v)
say 'callee v:' v 'callee zzz:' zzz
v = 'v-set-in-callee'
zzz = 'zzz-set-in-callee'
return
```
→
```
callee v: zzz callee zzz: z-in-caller
caller v: v-set-in-callee caller zzz: zzz-set-in-callee
```
The callee reads `v` as `zzz` (the caller's value), so `v` itself is exposed; and `ZZZ` is exposed too. Both halves, as the brief states. `run.rs`'s `DROP (v)` arm's own splitting and validation functions (`split_indirect_words`, `validate_indirect_word`) are reused rather than reimplemented, so the "upcase only after validating, one word at a time" rule is shared.

An indirect selector naming a stem also works: `sel = 'ST.'` / `procedure expose (sel)` → the callee's `st.1 = 'changed'` is visible in the caller.

### 3c. The five stem transcripts (`t5probe1`) — all reproduced exactly

| program | oracle | this crate |
|---|---|---|
| callee `a.1 = 'changed'` | `changed` | same |
| callee `a. = 'wiped'` | `wiped` | same |
| callee `drop a.` | `A.1` | same |
| `keep. = a.` first, callee `drop a.` | `A.1 orig` | same |
| callee reads `a.1` and an unexposed `other.1` | `callee reads: from-caller OTHER.1` | same |

### 3d. `USE ARG` (`t5probe1`, `t5probe2`, `t5probe3`)

Every value in the brief reproduced, plus what the brief did not state:

* `call sub 1,2,3` / `use arg p` → `[1]`, rc 0.
* `use strict arg p` with three arguments → rc 216, `Error 40.4: Too many arguments in invocation of SUB2; maximum expected is 1.`
* `call sub 1,,3` / `use arg p, q, r` → `[1] [Q] [3]`.
* `use arg >q` given a plain symbol → rc 168, 88.928. `call sub2 >caller` → the callee's `q = 'aliased'` makes the caller read `aliased`.
* **New:** `use strict arg p, q` with one argument → rc 216, `Error 40.3: ... minimum expected is 2.`
* **New:** a default satisfies STRICT's minimum — `use strict arg p, q = 'dflt'` with one argument runs, `[1][dflt]`.
* **New:** a trailing `...` suppresses the *maximum* check only — `use strict arg p, q, ...` takes four arguments.
* **New:** a default fills an omitted middle position — `call sub2 1,,3` / `use arg p, q = 'dflt', r` → `[1][dflt][3]`.
* **New, and it changes the implementation:** an absent target with no default is **dropped**, not left alone. With `preset = 'preset-value'` in the caller and a callee *without* `PROCEDURE`, `use arg p, preset` given one argument makes both the callee and the caller read `PRESET`. A probe whose target held a value equal to its own derived name could not have seen this.
* **New:** an *omitted* position for a `>` target is a **different** error from a wrong-kind value — rc 168, `Error 88.931: Argument 2 was omitted. A VariableReference argument is required.`
* `use arg st.` binds a stem target; `>st.` / `use arg >q.` aliases a whole stem.
* An aliased but unset variable reads as the **callee's** own derived name: `call sub2 >unsetvar` / `use arg >q` → `Q`, not `UNSETVAR`.

**One correction to the brief.** It records 88.928's text as `... found "caller".` for `call sub p`. `&2` is the argument's **rendered value**, not any spelling. My first probe could not tell the difference because it named the variable `caller`; three that do discriminate, all rc 168: `zebra = 'orig'; call sub2 zebra` → `found "orig"`; a literal argument `'literal-value'` → its own text; passing it second → `The 2 argument`. Re-running the original probe confirms `found "orig"`. The brief's transcript is consistent with this once its probe is read as having held the string `caller`.

### 3e. `PROCEDURE` placement, and `USE LOCAL` (`t5probe2`, `t5probe3`)

`PROCEDURE`'s legality is a **run-time** property, not a static one — which is why Step 2's "precompute each body's `PROCEDURE`/`EXPOSE` list" is not what I built (see §5). All rc 239, all `Error 17.1: PROCEDURE is valid only when it is the first instruction executed after an internal CALL or function invocation.`:

* at top level, no call at all;
* fallen into (`say 'main'` / `sub:` / `procedure`) — the *same* label text that works when called;
* after a `nop` in a called routine;
* inside `sub: interpret "procedure"`, i.e. a fragment does **not** inherit its host clause's permission.

And the one that shapes the mechanism: **two labels** between the `CALL` and the `PROCEDURE` (`sub:` / `lbl2:` / `procedure`) is legal. So a label neither grants the permission nor spends it, while a `NOP` spends it.

`USE LOCAL` measurements: first instruction of a program → rc 158, 98.993 `The USE LOCAL instruction may only be used from method invocations.` Second instruction of a program, first instruction of a called routine, and after a `PROCEDURE` → rc 157, 99.910.

### 3f. The variable reference in ordinary positions (`t5probe2`)

`p = 'orig'; say >p` → `orig`. `call sub2 >p` into a plain `use arg q` → `[orig]`, and the caller's `p` is untouched. `q~class` after `use arg >q` is `The String class` — the callee's variable is an ordinary variable, not a reference object. So `>` **decays to the referenced variable's value** everywhere except a `USE ARG >` target.

One thing I got wrong while writing the corpus program and caught by running it: `say 'text' >pref` is a **comparison** (prints `0`), because a `>` following a complete term is the operator. Only a leading `>` builds a reference. The corpus program and its header were corrected; `say >pref` stands alone.

---

## 4. Verification

### 4a. Differential against the oracle

A script comparing the three descriptors separately over every probe program: **50 of 62 byte-identical**. All 12 differences are accounted for, and **11 of them were verified byte-identical to the baseline binary built at `32a667ac`** — this task changed none of them:

* `q4`, `r1`, `t1`, `d1`, `d2`, `d3`, `inline` (7): `USE LOCAL` rejected by `rexx-parse` at parse time with the correct 99.910, rendered through `execute`'s parse-error arm at rc 120 instead of the oracle's three-line report. That is the standing, documented limitation at `crates/rexx-exec/src/lib.rs:1540` ("Parse errors remain deliberately not reproduced byte for byte").
* `i1`, `i2`, `i3` (3): `USE LOCAL` inside `INTERPRET` → **99.915** `INTERPRET data must not contain USE LOCAL.` — right number, right rc (157), both error lines byte-identical; only the fragment's own clause echo is missing, same parse-error limitation. The control `interpret "say 1/0"` matches the oracle byte for byte, so the echo machinery itself is fine.
* `w5` (1): `q~class`, a Phase 5 message send. It moved *forward* — at baseline it failed on `>x`, now on `~class`.

The twelfth, `c1`, did change, deliberately: `procedure expose a.1` was `PROCEDURE is not implemented (4b)` and is now `PROCEDURE EXPOSE of the single compound tail "A.1" is not implemented`. Still rc 120, still loud, a narrower and truer gap. See §5.

### 4b. Test output

```
cargo test --workspace   →  919 passed; 0 failed; 4 ignored   (baseline 901/0/4)
REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus
    mode: STRICT (the gate) -- REXX_CORPUS_GATE is set
    36 of 36 matching
assertions: 4224 of 4259 matching   (unchanged from baseline)
cargo fmt --all --check                                  → exit 0
cargo clippy --workspace --all-targets -- -D warnings     → exit 0
```

All 34 pre-existing corpus programs still match; two were added, `lang/call_procedure_expose.rex` and `lang/use_arg_forms.rex`, both byte-identical to the oracle. No pre-existing expected-byte literal changed. `crates/rexx-parse/tests/sourceline_oracle/` needed an expectation file for each new program; I generated them with the documented driver's **`LINEIN` fallback path only**, never `.Package~new` on a repository file, and then verified the result directly — the line count equals `wc -l` and the captured lines are byte-identical to the source files.

### 4c. Mutation testing — the new tests can fail

Because "a test that cannot fail is a defect here":

| mutation | tests run | result |
|---|---|---|
| `slot_ref` returns `SlotRef(frame.start + index)` (no chase) | `exposure_is_transitive…` | **FAILED** |
| same | `one_procedure_can_expose_names_living_in_two…` | **FAILED** |
| `PROCEDURE` reuses the caller's frame instead of pushing one | `procedure_isolates_and_expose_aliases…` | **FAILED** |
| absent `USE ARG` target left alone instead of dropped | `use_arg_drops_a_target_with_no_argument…` | **FAILED** |

All reverted; suite green afterwards.

Two of my own tests were wrong when first written, and both were caught by running them rather than by reading them. The frame-release test compared slot counts, which move for a reason that is not a leak — the first `CALL` in a program that never writes `RESULT` grows the top frame by one to hold it, and the error path never reaches that write, so the two paths had different baselines. It now counts **frames** (`RootSet::live_frames`), which has no such confounder, and carries a shared-pool control so that an implementation which never pushed a callee frame at all would not pass. The `USE LOCAL` test asserted 99.910 under `run_source`, which drives a body through `run_bounded` and never grants the first-instruction permission — it would have passed against an implementation with the two errors swapped. It now goes through `run_program`, the real entry point, and asserts rc 158 and the oracle's own message.

---

## 5. Two deviations from the brief's steps, both forced by measurement

**Step 2 ("precompute each body's `PROCEDURE`/`EXPOSE` list in `plan.rs`") is not what I built, and `plan.rs` is unchanged.** The brief's reasoning is *"`PROCEDURE` must be a routine's first instruction, so the list is a property of the body."* The first half is true of the grammar and false of execution: the same `sub: procedure` runs when called and raises 17.1 when fallen into (§3e), so no property of the body's text can decide it. The decision is therefore made at `PROCEDURE` execution time, from `Activation::first_instruction_pending` and `entered_by_call`, and this is also what preserves the `grow_slots` invariant (§2). "The isolation decision inside `CALL`" becomes: `CALL` marks the activation as callable-into, and `PROCEDURE` does the rest.

**`stem.rs` needed no functional change, only comments.** The brief says "not comments only. Seven of the twelve sites that resolve a frame from the top activation are here" — but that "seven of twelve" is the stale pre-activation-stack figure the brief itself flags. Under the `RootSet` route those seven sites are correct untouched, because the redirect is applied below them. I did not take this on trust: the five stem transcripts (§3c) all pass, including both `drop` cases, and the two `drop` ones were additionally re-run under the `stem_drop`→`clear_slot` mutant. `stem.rs`'s change in this commit is I17's write-up.

**One thing I could not do as specified.** Step 8 asks for I17's reclassification to be written into `mutate-4b.sh`'s comment. **There is no `mutate-4b.sh` in the tree** — only `rust/scripts/mutate-4a.sh`, which contains no stem mutation. I wrote the mechanism into `stem_drop`'s own doc comment instead, where a future `mutate-4b.sh` author will look, with an explicit note to copy it there and drop the mutant rather than list it as uncaught.

I verified I17 myself rather than restating the brief. `a.1='orig'; keep. = a.; drop a.; say a.1 keep.1; a.2 = 'second'; say a.2` prints `A.1 orig` / `second` under the oracle, under this code, **and under the mutant** — built and run both ways. The two `drop a.` stem-exposure transcripts also still print the oracle's `A.1` and `A.1 orig` with the mutant applied, so exposure does not open a gap either. The reason the two agree is that a stem's slot is never read as "empty or not": `read_stem` vivifies on a miss, so a cleared slot and a fresh empty stem are indistinguishable at every observation point this phase has. The mutant is genuinely equivalent; no distinguishing program exists, as the brief says.

---

## 6. Concerns

1. **`PROCEDURE EXPOSE` of a single compound tail is a new, disclosed loud gap.** Measured: with `a.1 = 'kept'` and `a.2 = 'other'` in the caller, `sub: procedure expose a.1` writing both tails leaves the caller printing `changed other` — tail 1 shared, tail 2 the callee's own. That is aliasing *inside* a stem object at one tail; 4b's mechanism aliases whole slots, and the stem lives in a slot while its tails do not. Exposing the whole stem instead would silently share `a.2`, so it fails loudly through `Loud::compound_expose`. This is not in the brief's scope and no corpus program needs it, but it is a real construct that now has a named gap rather than an approximation. It carries no owner string because nothing has been scheduled to build it.

2. **The `USE LOCAL` 99.910 arm is unreached and carries no test.** `rexx-parse` enforces the placement rule at parse time, so every shape that would take it fails first — eight tried, all intercepted (§3e, §4a). I kept the arm on the same reasoning `Loud::missing_body` states for its own unreached arm: answering 98.993 unconditionally would be a silent wrong answer the day that check moves. But it is unexercised, and a test for it would necessarily pass through the parse-time path and so could not fail if the arm were wrong. Flagged rather than papered over.

3. **The stem-heavy 2 % is a small-signal measurement.** Consistent in direction across three rounds, but within-round noise is comparable to the effect. If someone later needs a precise number, it wants a proper harness rather than wall-clock medians.

4. **`USE ARG` targets are limited to variable/stem/compound spellings.** The grammar admits a message term (`parseVariableOrMessageTerm`); that falls through to the existing `Loud::expression` path rather than being approximated. No measurement was taken of what the oracle does with one, because message sends are Phase 5 throughout this crate.

5. **`USE ARG`'s strict minimum is computed as the position of the last target with no default.** This reproduces every measured case (`p, q` → 2; `p, q = 'dflt'` → 1; `p` → 1). I did not measure the inverted shape `use strict arg p = 1, q`, where a default precedes a non-default; the rule as implemented answers 2 for it, which is the reading consistent with the measured cases but is an inference rather than a measurement.

---

# Fix round 1

Commits: **`d3413cbd`** (the fixes) and **`50ba7aa1`** (a correction to `d3413cbd`, below). Five source files in all; no corpus program and no expectation file touched.

`cargo test --workspace` **921 passed / 0 failed / 4 ignored** (was 919; two tests added).
`REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` → `mode: STRICT (the gate)`, **`36 of 36 matching`**.
`cargo fmt --all --check` exit 0; `cargo clippy --workspace --all-targets -- -D warnings` exit 0.

All probes below run from `t5fix1/`, a directory created for this round, wrapped as required, three descriptors read separately.

## C1 — `USE ARG >name` now raises 98.995, and the rule is the measured one

Fixed in `bind_use_target` (`run.rs`), with the predicate in `Interp::target_is_uninitialised` and the constructor `Raised::variable_reference_not_uninitialised` (98.995, one substitution).

**I re-measured the rule rather than taking it from the review, and the review's characterisation is exactly right.** The `f3`/`g1` pair is what settles it:

| probe | shape | oracle |
|---|---|---|
| `e1` | shared pool, `q` holds `'q-orig'` | rc 158, 98.995 `"Q"` |
| `f1` | shared pool, `q` unset | rc 0, alias installed |
| `f2` | `q = 'local'; drop q` | rc 0 — `DROP` restores the state |
| **`f3`** | **`procedure expose q`, exposed `q` holds a value** | **rc 158, 98.995** |
| **`g1`** | **`procedure expose q`, exposed `q` unset** | **rc 0, `p: via-alias q: Q`** |
| `f4` | `use arg >q` twice onto one target | rc 158, 98.995 |
| `g2` | `sub: procedure` / `q = 1` / `use arg >q` | rc 158, 98.995 |

Exposure is identical in `f3` and `g1`; only the value differs. So the trigger is the value, and an exposure check would have been wrong — the review is right that without `g1` that is the likely mistake.

### The one-line fix is not sufficient, and three measured programs show it

The review's suggested body is `if self.roots.slot(frame, index).is_some() { raise }`. **That over-raises on stems in this crate**, because `read_stem` vivifies a fresh empty `Body::Stem` into the slot on a bare stem read (it must — object identity is observable through `b. = a.`), and `stem_drop` leaves exactly the same thing. A slot being `Some` is therefore not the same question as the variable being initialised. Measured on the oracle:

| probe | shape | oracle | one-line fix would give |
|---|---|---|---|
| `h3` | `say q.` (bare read) then `use arg >q.` | **rc 0** | 98.995 — wrong |
| `k2` | `q.1 = 'x'; drop q.` then `use arg >q.` | **rc 0** | 98.995 — wrong |
| `k3` | `drop q.` on a never-touched stem | **rc 0** | 98.995 — wrong |
| `g3` | `q.1 = 'local'` then `use arg >q.` | rc 158, names `"Q."` | correct |
| `k1` | `q. = 'dflt'` then `use arg >q.` | rc 158 | correct |

So: **a stem slot holding a stem with no default and no live tail counts as unset**, which is precisely what both the vivified read and the `DROP` leave. `stem.rs` gains `is_uninitialised_stem` for it.

**"No live tail" and not "no tails" — and the first version of this fix got that wrong.** `d3413cbd` shipped `tails.is_empty()` and disclosed it in this report as an inference: that a tombstoned tail counted as content, unmeasured. I then measured it, and it was wrong in the direction that refuses a legal program. Four further shapes, `50ba7aa1`:

| probe | shape | oracle | `tails.is_empty()` gave |
|---|---|---|---|
| `k5` | `q.1 = 'x'; drop q.1` | **rc 0** | 98.995 — wrong |
| `k8` | two tails, both dropped | **rc 0** | 98.995 — wrong |
| `k6` | `q.1='x'; q.2='y'; drop q.1` (one live tail left) | rc 158 | correct |
| `k7` | `q. = 'dflt'; drop q.1` (default survives) | rc 158 | correct |

A tombstone is a present key with **no value** — which is what `Body::trace` already tells the collector, "present but dead" — so whether the map has entries is the wrong question. `k6` and `k7` are what stop the correction overshooting into "ignore tails entirely". Both mutants are caught by the test: `tails.is_empty()` (too strict) and accepting any stem (too lax).

**And the exemption keys on the target's *name* shape, not the value's.** `k4`: `zz = q.` puts a fresh empty stem object into a *simple* variable, and `use arg >zz` then raises 98.995 naming `"ZZ"`. A check written as "the value is an empty stem" passes every row above and fails here. `ZZ` is an initialised simple variable that happens to hold a stem; `Q.` is a stem variable nobody has written.

`h1`/`h2` (a *compound* read `say q.1`, and a simple-variable read, before aliasing) are rc 0 on both — `stem_get` does not vivify, only `read_stem` does, so those needed nothing.

### Verification

All **17** `t5fix1` probes are byte-identical to the oracle on stdout, stderr and exit status.

Mutation checks:

| mutation | test | result |
|---|---|---|
| the 98.995 check removed entirely | `use_arg_alias_requires_an_uninitialised_target` | **FAILED** |
| the 98.995 check removed entirely | `use_arg_alias_treats_an_empty_stem_as_uninitialised` | **FAILED** |
| stem exemption removed (the review's one-line form) | `use_arg_alias_treats_an_empty_stem_as_uninitialised` | **FAILED** |
| `tails.is_empty()` instead of "no live tail" | `use_arg_alias_treats_an_empty_stem_as_uninitialised` | **FAILED** |
| tails ignored entirely | `use_arg_alias_treats_an_empty_stem_as_uninitialised` | **FAILED** |

The third confirms the exemption is load-bearing rather than decoration; the last two pin the tombstone rule from both sides.

Tests added, as the set the review asked for plus the stem boundary: `use_arg_alias_requires_an_uninitialised_target` (assigned-then-aliased **raises**; exposed-and-holding-a-value **raises**; exposed-but-unset **succeeds**; dropped **succeeds**; repeated alias **raises**) and `use_arg_alias_treats_an_empty_stem_as_uninitialised` (never-touched, bare-read, and dropped stems **succeed**; written tail and assigned default **raise**; simple variable holding an empty stem **raises**). The comments carry why the pair discriminates.

## I1 — the frame-release control's claim was false

**Reproduced before fixing.** I built the exact mutant the comment named — `exec_procedure` reusing `outer`, `owns_frame = false`, no `push_slots` — and `an_isolated_callees_frame_is_released_on_both_paths` passed under it, control included. All three blocks compare `live_frames()` against the same value `1`, so no frame count can catch a frame that is never pushed.

The claim is replaced by the property the block **does** carry, verified the same way rather than asserted: popping unconditionally instead of only when `owns_frame` tears down the caller's still-live frame. That mutant **fails** this block and **passes** `procedure_isolates_and_expose_aliases_the_caller_entry`, so this block is the one carrying it. The comment now also names the isolation tests as what catch the never-push mutant.

## M3 — the gap's disclosure widened

Verified: `v = 'A.1'` with `procedure expose (v)` reaches `Loud::compound_expose` — oracle rc 0 printing `changed other`, this crate rc 120. `expose_names` expands the selector's value into ordinary names and a compound-shaped word among them arrives at the same check. `compound_expose`'s doc now states both spellings. Report §6 concern 1 should be read with that addition.

## M1 — corrected rather than deferred, because this round gave it a new reason to be false

M1 was deferred on the review's expectation that *"both become true once C1 lands"*. **With the stem exemption they do not quite.** `USE ARG >q.` may alias over a vivified-but-empty stem, so an aliasing slot's own entry is guaranteed to hold nothing *reachable*, not to be `None`. Since that is a falsehood this round introduced, I corrected both sentences instead of leaving them:

* `alias_slot`'s doc now states what its two callers actually guarantee (a freshly pushed frame's slots are all `None`; `USE ARG >name` refuses an initialised target) and names the stem exception.
* `iter`'s doc keeps the load-bearing conclusion — the exposed value is yielded exactly once, never zero times — and replaces the "stays `None` for its whole life" sentence with the bounded cost: one retained empty stem per such alias, over-retention only, never a collected live object.

M2 is untouched and remains for the whole-branch review.

## Concerns

1. **I shipped one inference in `d3413cbd` and it was wrong.** The tombstone case was disclosed as unmeasured with a claim that the direction was conservative; measuring it showed the opposite — it refused a program the oracle accepts. Corrected in `50ba7aa1`. Worth recording as the pattern rather than the instance: the "conservative direction" argument was doing the work that a two-minute probe should have, on a rule I had already measured eight neighbouring shapes of.
2. **The compound-expose gap is unchanged in size**, only in disclosure. Both spellings fail loudly; neither is approximated.
3. ~~`USE ARG >name` onto an exposed *and* already-aliased target is untested.~~ **Closed by measurement**, rather than left as a second inference in the same round:

   * `n1` — `procedure expose q`, `use arg >q`, then `use arg >q` again: oracle rc 158, 98.995; crate byte-identical.
   * `n2` — `procedure expose q`, `use arg >q`, then `q = 'written'`: oracle rc 0, `p: written q: Q`; crate byte-identical. The write goes through the `>` alias to the caller's `p`, and the caller's own exposed `q` stays unset — so `USE ARG >name` *replaces* the exposure alias rather than chaining through it, which is what `alias_slot` overwriting the entry does.

   No concerns outstanding from this round beyond the two above.

---

# Fix round 2 — the reference-kind refusals

Commit: **`92e8021816f7c985199fe302e44cb960e0821413`** (`92e80218`). Three source files (`run.rs`, `error.rs`, `lib.rs`); no corpus program and no expectation file touched.

`cargo test --workspace` **923 passed / 0 failed / 4 ignored** (was 921; two tests added).
`REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` → `mode: STRICT (the gate)`, **`36 of 36 matching`**.
`cargo fmt --all --check` exit 0; `cargo clippy --workspace --all-targets -- -D warnings` exit 0.

Probes from `t5fix2/`, a directory created for this round, wrapped as required, three descriptors read separately.

## The measured matrix

**Fourteen cells, all measured; none inferred.** The five team-lead listed are marked ★.

| # | caller passes | callee target | oracle | crate before | crate now |
|---|---|---|---|---|---|
| ★ | plain symbol | `>q` simple | 88.928 rc 168, subst = **value** | correct | correct |
| ★ | `>p` simple ref | `>q.` **stem** | **88.929** rc 168 | **rc 0, silent** | correct |
| ★ | `>p.` stem ref | `>q` **simple** | **88.930** rc 168 | **rc 0, silent** | correct |
| ★ | `>p.` stem ref | `>q.` stem | rc 0 | correct | correct |
| ★ | `>p` simple ref | `>q` simple | rc 0 | correct | correct |
| a5 | plain symbol | `>q.` **stem** | **88.928**, subst = **value** | correct | correct |
| b3 | literal `'literal'` | `>q.` stem | 88.928, subst = value | correct | correct |
| b4 | omitted | `>q.` stem | 88.931 | correct | correct |
| b1 | `>p` at position 2 | `>q.` stem | 88.929, `The 2 argument` | rc 0, silent | correct |
| b2 | `>p`, `USE STRICT ARG` | `>q.` stem | 88.929 — strict changes nothing | rc 0, silent | correct |
| c1 | `>p` simple ref | `>q.` stem, **already assigned** | **88.929**, not 98.995 | rc 158 (98.995) | correct |
| c2 | `>p.` stem ref | `>q` simple, **already assigned** | **88.930**, not 98.995 | rc 158 (98.995) | correct |
| c3 | `>r` after a prior `use arg >q` | `>q.` stem | 88.929 naming `R` | rc 0, silent | correct |
| b5 | `>p.1` compound ref | `>q` simple | parse-time **20.930** rc 236 | — | — |
| b6 | `>p` simple ref | `>q.1` compound | parse-time **20.931** rc 236 | — | — |

The last two are `rexx-parse`'s, raised at `expr.rs:980`/`instruction.rs:1874` — code this round does not touch — and land under the standing parse-error rendering limitation (right numbers, rc 120 instead of 236), unchanged and pre-existing. They matter here for a positive reason: **compound is not a third kind to handle**, because neither side can be compound in a program that runs.

## Three findings that shaped the code rather than confirming it

1. **88.929/88.930 substitute the *caller's* variable name; 88.928 substitutes the argument's *value*.** Probed with `p = 'value-not-name'` so the two cannot be confused — `>p` into `>q.` reports `found "P"`, while a plain `p` into the same target reports `found "value-not-name"`. This is the identical probe defect this task hit once already on 88.928 (a variable named `caller`), so it was designed against this time. A stem reference reports `P.` with its period, which is `use_target_name`'s own output and needs no shaping.
2. **The kind check runs before the uninitialised check.** `c1`/`c2`: a target that is both kind-mismatched and already assigned reports 88.929/88.930, not 98.995. Getting this backwards is a different number *and* a different rc, so it is a wrong answer rather than differently-worded right one.
3. **`USE STRICT ARG` does not change the kind rule** (`b2`), and the position substitution is the argument's own (`b1`).

`Argument::Reference` now carries the referenced variable's own spelling — both the kind (its shape) and the substitution (its text) — so `Argument` is `Clone` rather than `Copy`. One small allocation per `USE ARG >` position, none for an ordinary argument.

## Verification

All **12** runnable `t5fix2` probes byte-identical to the oracle on stdout, stderr and exit status; `b5`/`b6` are the pre-existing parse-time pair above.

Mutation checks — each caught by both new tests unless noted:

| mutation | result |
|---|---|
| kind check removed entirely | **FAILED** (both tests) |
| always refuse a stem target | **FAILED** (both tests) |
| 88.929 and 88.930 swapped | **FAILED** (both tests) |
| uninitialised check moved before the kind check | **FAILED** (ordering test; kind test passes, correctly — it does not exercise ordering) |

"Always refuse a stem target" is the one team-lead's pairing rule exists to catch, and the passing `>p.` → `>q.` cell is what catches it.

**A process note worth recording.** My first two attempts at the ordering mutant reported "passed", and both were wrong: the string surgery had not applied, because comment lines sit between the two blocks. I only found that by printing the mutated region instead of trusting the script's exit. A mutation that silently fails to apply reports the test as *not* discriminating in the one direction that looks like good news — the test appears to pass under the mutant, which reads as "already covered elsewhere" rather than "check your edit". Verifying the mutant is applied is part of the mutation check, not preamble to it.

## Concerns

1. **`ARG()` will need the same argument list and may want different data.** `CallContext::arguments` now carries a name on every reference, sized for `USE ARG >`. 4c's `ARG()`/`ARG(n)` reads the same list and only wants values; nothing is wrong today, but whoever lands it should check that a reference argument reports what the oracle says it reports rather than assuming `Argument::value()` is the whole answer.
2. **The kind rule is expressed as a two-way `shape_of` comparison**, which is exactly right while the kinds are {simple, stem}. If a later phase makes a compound reference reachable — both sides are parse-rejected today — this becomes a three-way question and the `!=` stops being adequate. Stated at the check.
