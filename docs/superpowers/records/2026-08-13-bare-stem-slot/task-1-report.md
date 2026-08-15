# Task 1 -- a bare stem's slot, taken

Plan: `docs/superpowers/plans/2026-08-13-bare-stem-slot.md`.
Base `50faeff88`. Head **`96d7a87a8`** (the change) and **`866d07d1b`** (record entry 32), both read back from `git log`.

## What this task found that the plan did not predict

**The obvious way to take the third site makes every controlled loop in the language slower, and the workload the plan asked for is what caught it.**
Forwarding the loop's kept `at` from `bind_control`'s stem arm into `Interp::assign_expr_target` -- the shape the plan's Step 2 describes, and the one I built and measured first -- costs **2 instructions on every pass of every controlled loop**, including loops with a simple control variable that never enter the stem arm at all. Why the generated code changes was not established; the effect was, on three programs and by partial revert. `emptyloop` +50,000,012 and `varlookup` +38,000,036, against same-binary spans under 1,500.
It is not layout noise: reverting that one line and nothing else puts `emptyloop` back on base exactly, and the movement is precisely +2 per pass on three different programs.

The design that shipped takes the slot from the plan's own `CompoundName` entry instead, so `bind_control`'s call site keeps the compile-time `None` it had. It keeps 97% of the stem-loop saving, costs nothing on any axis, and works on **both** engines where the op route would have worked on one.

## The change

`Plan::bind` already gave an `ExprKind::Stem` symbol's spelling and its id one slot; a stem-shaped name has no stem half distinct from itself, so that slot **is** the stem's slot. `bind` now records it on the entry it was already building, and the two bare-stem writes that pay read it back:

| file | what changed |
|---|---|
| `plan.rs` | `bind` fills `CompoundName::stem_at` for a stem-shaped name (and still leaves it `None` for a compound-shaped one, where the stem is a second name the plan never bound) |
| `run.rs` | `assign_expr_target`'s `ExprKind::Stem` arm takes the slot off `Code::compound`; the controlled loop's re-test does the same for its bare-stem read |
| `stem.rs` | `stem_assign` -> `stem_assign_at`, `replace_stem` gains the slot, both deciding it in the existing `Interp::stem_slot` |

`control_slot`, `write_slot`, `Op::Store` and the `drive.rs` tripwire are **unchanged** from base. Six doc comments were corrected.

### The three sites the plan named

* **`stem_assign`** -- taken, through `assign_expr_target`'s stem arm, on both engines.
* **`replace_stem`** -- taken, through `stem_assign_at`. Its other caller, `stem_drop`, still passes `None`: `drop_by_name` serves an already-upcased `DROP` target and every word of an indirect subsidiary list from a plain string, with no id in hand.
* **`bind_control`'s `NameShape::Stem` arm** -- taken, but **not** by carrying `control_slot`'s answer. See above; `control_slot`'s doc now carries the measurement.

### A fourth site, in the same loop, not in the plan

The controlled loop's own **re-test read** (`run.rs`, `NameShape::Stem => self.read_stem(name)`) is reached on every pass of a stem-controlled `DO`, immediately beside site 3's write. Left alone, it would have halved the win and left a fifth task. Measured by partial revert: it is worth **867,001,000** instructions of `stemloop`'s 1.659 billion, and it costs nothing on `emptyloop`.

## Step 1: the workload, written and measured before the fix

Two programs, in the session scratchpad, quoted here in full so they can be rebuilt:

```rexx
/* stemloop.rex -- a stem-controlled DO, the only shape that reaches
   bind_control's Stem arm, which writes the control variable every pass. */
n = 3000000
do zs. = 1 to n
  nop
end
say 'done'
```

```rexx
/* stemwrite.rex -- a bare stem write in a loop body: assign_expr_target's
   Stem arm, per statement, the shape rexxcps writes fourteen times an
   iteration. */
n = 6000000
do i = 1 to n
  zs. = i
end
say 'done'
```

Both were checked against the oracle before being used: `do zs. = 1 to 3` with `say zs.`/`say zs.1` inside and after it, and `zs. = 'w'` after, is byte-identical between the oracle and this crate on **both** engines (`4`, `w w`, rc 0, empty stderr).

### Neither belongs in `rust/bench-programs/`, and that is a decision

**No.** Both stay task-local, and the record entry carries their text the way entry 29 carries the `INTERPRET` cost program.

* A **stem-controlled `DO`** is close to nonexistent in real Rexx: the control variable is a whole stem, and the loop writes the stem's *default* on every pass. Making it a permanent axis would give it a vote in every later entry that its share of real programs cannot justify -- and a later task optimising against it would be optimising against a construct nobody writes.
* A **bare stem write in a loop** already has an axis: `samples/rexxcps.rex` writes `avar.=1.0''loop` fourteen times per iteration, which is the same shape at a realistic frequency. `stemwrite.rex` is that shape with everything else removed, which is a diagnostic instrument rather than a workload.

What both are good for is exactly what they were used for here: making a per-iteration effect large enough to read, and making a per-iteration *regression* large enough to catch. That job is done at the moment the entry is written.

### The base, and each axis's own spread, measured before anything changed

`perf stat -e instructions:u`, `REXX_ENGINE=ir`, six runs of the one base binary staged at one fixed path, from a fresh empty directory.

| axis | base, minimum of six | same-binary span |
|---|---:|---:|
| `stemloop` | 11,402,819,719 | 1,236 |
| `stemwrite` | 13,379,958,361 | 1,076 |
| `compound` | 25,053,533,979 | 2,901,211 |
| `alloc4c` | 8,189,889,341 | 50,817 |
| `rexxcps` | 25,298,097,696 | 6,448,443 |
| `arith` | 20,416,193,212 | 3,216 |
| `strings` | 44,928,791,862 | 36,000,462 |
| `varlookup` | 44,840,840,285 | 1,508 |
| `emptyloop` | 27,150,813,244 | 622 |

`strings`' span is one run 36 million high -- the same interference signature entry 29's reviewer and entry 30 both recorded on that axis, seen here on the base arm before any change existed.

## Step 3: the `extra` question, answered by running -- **yes, a fourth time**

`Interp::slot_of` instrumented to print which of its three sources answered, release build, fresh empty directory, **both engines**, at **head**.

| program | what `slot_of` printed |
|---|---|
| `interpret "zr. = 7"` | `grow-into-extra "ZR."`, then `hit-extra "ZR."` |
| `interpret "do zt. = 1 to 3; nop; end"` | `grow-into-extra "ZT."`, once, and no pass resolves it again |
| `zn = 'ZU.'; drop (zn)` | `grow-into-extra "ZU."` |
| `zk = value('ZW.', 'set')` | `grow-into-extra "ZW."`, then `hit-extra "ZW."` |
| **control** `zs. = 'one'; say zs.` | **nothing at all** under `ir`; `plan "ZS."` twice under the tree-walker |
| **control** `call zsub`, `use arg zy.` | `plan "ZY."` |

So a bare stem **can** be bound in the activation's `extra` rather than in the plan, by at least four routes, and `stem_at`'s `Option` is required rather than defensive. The rows are head-arm runs and are labelled as such; the control row is the interesting one at head, because silence is direct evidence that the precomputed slot is being used, which no test in the workspace can see (mutation P3 below).

**A new shape this task had to find: the fragment.** `interpret "do zt. = 1 to 3; end"` builds a fragment plan whose ids are its own, translated into the enclosing frame through `slot_of`, which grew `ZT.` into `extra`. My **first** design routed that translated slot into `Interp::stem_slot`, whose tripwire compares against the plan's map alone -- and it fires there. Measured, in a debug build: the old tripwire fires on exactly that one program of the seven probes, on both engines, and **the whole workspace suite is green with it**, so nothing in the tree reached that shape. The shipped design does not route a fragment's slot anywhere (`Code::plan` is `None` for a fragment, so `Code::compound` answers `None`), which is why the tripwire is unchanged from base -- but the shape is now covered by a test, because it was a live latent panic in the design I nearly shipped.

## Step 4: the answers did not move

`Interp::stem_slot`'s permanent `debug_assert_eq!` is the accessor-level check the plan asks for, and it is the same one the previous task landed. At the final state of this commit, whole workspace, `--no-fail-fast`, under `memcap 8G`: **1483 passed, 0 failed, 4 ignored, zero firings.**

**Inverted to `debug_assert_ne!`, it fires: 1445 passed, 38 failed, 37 distinct test names.** The zero is a live zero.

### Mutations

Whole workspace, `--no-fail-fast`, `memcap 8G`, restored from a `cp -p` backup and `cmp`-verified after each, rebuilt, and the suite re-run at the restored final state.

| mutation | result | distinct failing names |
|---|---|---:|
| `stem_slot`'s tripwire inverted to `debug_assert_ne!` | 1445 passed, 38 failed | 37 |
| **P1** `bind` records `None` for a stem-shaped name | 1482 passed, **1** failed | 1 |
| **P2** `bind` records `Some(slot + 1)` | 1461 passed, 22 failed | 21 |
| **P2'** the same, tripwire deleted | 1469 passed, 14 failed | 14 |
| **P3** `assign_expr_target`'s stem arm ignores the entry | 1483 passed, **0** failed | 0 |
| **P4** the loop re-test's stem arm ignores the entry | 1483 passed, **0** failed | 0 |
| **G** `note_compound_name` records `Some(slot + 1)` | 1447 passed, 36 failed | 35 |
| **G'** the same, tripwire deleted | 1454 passed, 29 failed | 28 |
| **H2** `write_slot` answers for a compound target too | 1483 passed, **0** failed | 0 |
| **I2** `control_slot` answers for a compound control too | 1483 passed, **0** failed | 0 |

* **P1 is caught by exactly one test**, `plan::tests::bind_keeps_a_stem_shaped_names_own_slot_as_its_stems`, which is the test written for it. Nothing else in the workspace can see the slot not being taken, because taking it and resolving it produce the same answer -- which is the point, and which P3 and P4 restate from the other end.
* **P2 minus P2' is seven tests**: a wrong bare-stem slot is caught by seven tests **only** because the tripwire is there. So the tripwire is a measured net catcher for this source too, not only for the compound source the previous task measured it on. I ran the pair rather than importing that result.
* **G minus G' is the same seven**, and the same seven names the previous task recorded. The tripwire's catching power on the compound source is unchanged by this task, re-measured at the final state rather than carried over.
* **H2 and I2 are the two claims `ir/drive/tests.rs` makes about the filters being unobservable**, and they were re-run here because the earlier form of that comment was measured against a tree in which `write_slot` answered for a stem, and stopped being true when it did not.

## Step 5: the measurement, and the profile

### Instructions

`perf stat -e instructions:u`, `REXX_ENGINE=ir`, six interleaved rounds per axis, **both arms staged at one fixed path** so `argv[0]` is byte-identical, from a fresh empty directory. Minimum of each arm.

| axis | base `50faeff88` | head | difference | | span base | span head |
|---|---:|---:|---:|---:|---:|---:|
| `stemloop` | 11,402,819,545 | 9,743,819,810 | -1,658,999,735 | **-14.55%** | 1,573 | 835 |
| `stemwrite` | 13,379,958,659 | 11,729,958,718 | -1,649,999,941 | **-12.33%** | 1,227 | 834 |
| `rexxcps` | 25,298,108,095 | 25,259,319,732 | -38,788,363 | **-0.153%** | 6,428,894 | 7,160,383 |
| `varlookup` | 44,840,840,190 | 44,802,839,869 | -38,000,321 | -0.085% | 495 | 1,576 |
| `compound` | 25,052,694,664 | 25,028,455,741 | -24,238,923 | -0.097% | 1,599,903 | 2,598,819 |
| `strings` | 44,928,791,694 | 44,913,791,298 | -15,000,396 | -0.033% | 1,025 | 1,354 |
| `alloc4c` | 8,189,876,434 | 8,183,867,131 | -6,009,303 | -0.073% | 117,496 | 92,743 |
| `arith` | 20,416,193,212 | 20,413,693,451 | -2,499,761 | -0.012% | 924 | 746 |
| `emptyloop` | 27,150,812,848 | 27,150,812,765 | -83 | -0.000% | 1,348 | 752 |

Every `rexxcps` run of both arms self-calibrated to `100 x 100`, checked on each arm's own `Averaged:` line. Every axis's six runs are non-overlapping between arms except `emptyloop`, whose -83 is inside both spans and is a **bound, not a difference**.

**No axis moved up.** That is unusual for this family and it is not claimed as a benefit of the change beyond the two axes it can reach.

### What is attributable and what is not

An accessor-level probe -- `stem_assign_at` and `read_stem_at` instrumented to print -- was run over every axis on both engines. It answers identically on both:

| axis | bare-stem operations | carrying a slot |
|---|---:|---:|
| `stemloop` | 6,000,001 | 6,000,001 |
| `stemwrite` | 6,000,000 | 6,000,000 |
| `rexxcps` | 140,000 | 140,000 |
| `compound` | 1 | 1 |
| `alloc4c`, `arith`, `strings`, `varlookup`, `emptyloop` | 0 | 0 |

* **`stemloop` and `stemwrite` are the change's own axes** and their movement is its semantics: 6,000,001 and 6,000,000 name resolutions removed, 1.659 and 1.650 billion instructions.
* **Five axes execute no bare-stem operation at all** and still moved by -83 to -38,000,321 instructions, thousands of times their own spans. **That is codegen drift, and I offer no attribution beyond "not this change's semantics"** -- separating drift from binary layout needs a do-nothing control this task did not build, and entry 31 is the standing correction for stating a cause where only a bound is earned. It happens to be favourable here; that is luck, not a result.
* **`rexxcps` is the interesting one and it is only partly attributable.** It executes 140,000 bare-stem writes -- `avar.=1.0''loop`, fourteen an iteration -- and moved -38,788,363 with non-overlapping arms. 140,000 removed `slot_of` calls on a five-byte name cannot be 38 million instructions; most of that figure is the same drift the compound-free axes show. **What is claimed is the direction and the bound**, not a per-write cost.
* **`compound` executes exactly one bare-stem operation** (`t. = 0`) and moved -24 million. Drift.

### The rejected design, measured

Kept here because it is the more valuable of the two results.

| axis | base | rejected head | |
|---|---:|---:|---:|
| `stemloop` | 11,402,819,601 | 9,695,819,491 | -14.97% |
| `stemwrite` | 13,379,958,909 | 11,693,959,045 | -12.60% |
| `rexxcps` | 25,298,098,536 | 25,268,707,828 | -0.116% |
| `emptyloop` | 27,150,813,233 | 27,200,813,245 | **+50,000,012** |
| `varlookup` | 44,840,839,859 | 44,878,839,895 | **+38,000,036** |
| `compound` | 25,052,692,974 | 25,063,534,582 | +10,841,608 |
| `strings` | 44,928,791,773 | 44,934,791,309 | +5,999,536 |
| `alloc4c` | 8,189,911,484 | 8,191,894,926 | +1,983,442 |
| `arith` | 20,416,193,078 | 20,417,193,335 | +1,000,257 |

`emptyloop` runs 25,000,000 passes and `varlookup` 19,000,000: **exactly +2 instructions per pass on both**, and `strings` at 3,000,000 iterations is +2 a pass to within its startup. Isolated by partial revert, three runs each, minimum:

| arm | `emptyloop` | `stemloop` |
|---|---:|---:|
| base | 27,150,813,500 | 11,402,819,653 |
| rejected head | 27,200,813,082 | 9,695,819,133 |
| head minus the re-test's `read_stem_at` | 27,200,813,496 | 10,562,819,715 |
| head minus `bind_control`'s forwarding | **27,150,813,214** | 10,559,820,233 |

So the +2 is `bind_control` forwarding a value where a compile-time `None` used to sit, and nothing else. Recomputing `control_slot(code, control)` inside the stem arm instead is **worse** -- `emptyloop` +100,000,000 (+4 a pass), `varlookup` +76,000,000 -- so the fix is not "avoid keeping `at` alive", it is "leave that call site's argument constant".

### The hashing bucket

`perf record -F 999`, `REXX_ENGINE=ir`, over `samples/rexxcps.rex`, both arms staged at the one fixed path, three runs per arm, one sitting, bucketed by self time. The bucket is named rather than counted: `hash_one::<&[u8]>`, SipHash's own `write`, `Interp::slot_of`, `hash_one::<&SymbolId>`, `hash_one::<&Vec<u8>>` and `HashMap<&str, ()>::contains_key::<str>`.

| | base `50faeff88` | head |
|---|---|---|
| **bucket** | **5.55% / 6.50% / 5.99%** | **6.25% / 5.95% / 7.14%** |
| `hash_one::<&[u8]>` | 1.72 / 2.30 / 1.71 | 2.02 / 1.65 / 2.62 |
| SipHash `write` | 2.13 / 1.81 / 1.75 | 1.90 / 2.20 / 1.94 |
| `Interp::slot_of` | 0.57 / 1.06 / 0.98 | 1.02 / 0.74 / 1.15 |
| `hash_one::<&SymbolId>` | 0.90 / 0.88 / 0.85 | 0.83 / 0.90 / 0.93 |
| `hash_one::<&Vec<u8>>` | 0.12 / 0.45 / 0.56 | 0.26 / 0.27 / 0.36 |

**The bucket did not fall, and the two arms' ranges overlap completely on it and on every member.** The instruction counter reads `rexxcps` down 38.8 million with non-overlapping arms; the sampled share cannot resolve that, because 140,000 removed name resolutions are a small fraction of a 25-billion-instruction program and the within-arm spread on this instrument is over a percentage point. **This is the honest answer and it is not a null result**: it says the sampled share is the wrong instrument for what is left, not that nothing was removed.

The brief's expected base figures (5.96 / 6.36 / 5.65%, `hash_one::<&[u8]>` 1.89-2.77%, SipHash `write` 1.49-1.89%, `slot_of` 0.72-1.41%) reproduce: my base arm reads 5.55-6.50% on the bucket, inside that range on every member.

### What is left in the bucket, and whether anything nameable remains

**Measured, not inferred.** `Interp::slot_of` instrumented at head and run over `rexxcps`:

```
3,080,203 slot_of calls, of which
  2,800,003  PARSE targets      (plan-resolved: A1..A4, B1..B3, C1..C3, P1..P8, V1, V2)
    140,199  RESULT             (through extra)
    139,999  SIGL               (through extra)
          2  one-offs at startup (SOURCE, SYSTEM, VERSION and their neighbours)
```

**Not one of them is a stem or a compound.** On this program, this family is finished.

What is nameable and remains:

1. **`PARSE` targets, and it is this family's finding one instruction further along.** `parse_template.rs` passes `None` for every target, and its own comment says why: "No compiler resolved a `PARSE` target". Measured directly, with the target's own arm instrumented, on both engines: **2,800,003 of 2,800,003** `PARSE` writes on `rexxcps` come back `by_symbol=Some(n)` with `slot_of` computing that same `n` -- `V2 Some(19)/19`, `P8 Some(28)/28`, and so on for every name. A three-line probe (`parse var zline za zb zc`) reads `Some(1)/1`, `Some(2)/2`, `Some(3)/3` on both engines. This is 91% of what is left, and it is the same shape as the four applications already landed. **The lesson from this task applies to it directly:** `parse_template.rs`'s call site passes a literal `None` today, and replacing that literal with a value is exactly what cost 2 instructions a pass here, so whoever takes it should measure `emptyloop` and `varlookup` before believing the win.
2. **`RESULT` and `SIGL`, 280,198 calls.** `Interp::set_sigl` and the `CALL` return path go through `assign_by_name` from a run-time byte string, and there is no symbol to hang a slot on. Removing these means giving the activation two dedicated fields or two well-known slots, which is a different kind of change from this family.
3. **`hash_one::<&SymbolId>`, 0.83-0.93%.** This is the *precomputed* side -- `by_symbol` and `Code::slots` are `HashMap<SymbolId, usize>`, so every slot this family saves is bought with a SipHash of a `u32`. It is now the same order as `slot_of` itself. A `Vec<Option<usize>>` indexed by `SymbolId::index()` would remove it outright, the way `Plan::compounds` already is.
4. **`hash_one::<&Vec<u8>>`, 0.26-0.56%.** The stem's own tails map, keyed by the resolved tail key. That is the data structure, not a symbol lookup, and no member of this family can touch it.

So: **the stem-and-compound family is finished on this path**, and what is left in the bucket is one clean successor (`PARSE` targets), one different-shaped problem (`RESULT`/`SIGL`), one cost this family introduced (`SymbolId`-keyed maps), and one irreducible (the tails map).

## Behaviour

* **Byte-identity across arms**, every axis, **both engines**: `stemloop`, `stemwrite`, `compound`, `alloc4c`, `arith`, `strings`, `varlookup`, `emptyloop` produce identical stdout, identical stderr and identical exit status under the base and head binaries. `rexxcps` differs only in its own two timing lines, with `100 x 100` on every arm.
* **The oracle**, on the shapes this change touches, wrapped `( ulimit -v 1048576; LD_LIBRARY_PATH=... )` from a fresh empty directory with absolute paths and `</dev/null`: a stem-controlled `DO` with reads inside and after it, a bare stem write and read, and a stem-controlled `DO` inside an `INTERPRET` with the observation also inside an `INTERPRET` -- all byte-identical to this crate on both engines.
* **The measured binary is the committed one.** Rebuilt from the sources at the final state and `sha256sum`'d: identical to the binary every figure above was taken with.

## Gates

From `rust/`, at the final state, each status read unpiped:

| gate | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, `Checking rexx-exec` in the log |
| `memcap 8G cargo test --workspace --no-fail-fast` | exit 0, **1483 passed, 0 failed, 4 ignored** |

Baseline was 1481. The two new tests are `plan::tests::bind_keeps_a_stem_shaped_names_own_slot_as_its_stems` and `run::tests::a_stem_control_inside_a_fragment_writes_the_enclosing_frames_slot`.

## The comment sweep

The plan's second defect class asks for the claim's vocabulary to be swept, not just the instances a review named. I swept for `no slot`, `not a slot`, `nothing to read`, `left on the table`, `nothing reads it`, `resolves by name`, `through a name rather than`, `not carried`, `bare stem`, `stem target`, `stem_assign`, `control_slot` and `write_slot` across the crate tree with `/bin/grep -a`.

**Corrected** (all were instances of the claim this task falsifies):

* `ir/mod.rs`, `PlanSlot`'s doc -- said a stem write's `UNRESOLVED` "records that nothing downstream reads it yet". Something reads it now.
* `ir/compile.rs`, `write_slot`'s doc -- said a stem target is unresolved "because nothing downstream uses it yet".
* `run.rs`, `control_slot`'s doc -- said `None` "records that nothing reads it yet".
* `run.rs`, `bind_control`'s doc and its stem arm's comment.
* `run.rs`, `assign_expr_target`'s doc -- said using the stem's slot "would be an optimisation, and it wants its own measurement".
* `plan.rs`, `CompoundName::stem_at`'s doc and `bind`'s doc -- both said `bind` assigns the stem nothing.
* `lib.rs`, `Code::stem`'s doc -- same.
* `ir/drive/tests.rs`, `a_compound_control_resolves_its_tail_on_every_pass`'s doc -- claimed it catches `write_slot` resolving every target shape, and gave the reason as `at` being read in `bind_control`'s `Simple` arm alone. Both were re-measured rather than reasoned about, and the paragraph now states what the runs said. (It also carried "and eight others", a count of a mutable repo aggregate, which is gone.)
* `plan.rs`, `build_records_a_compounds_split_under_the_compounds_own_id`'s doc.

**Read and left alone, as not-instances:**

* `eval.rs:138` "a compound is the kind a compiled read has no slot for" -- about compounds, still true.
* `ir/golden_tests.rs:335` "`ZA.ZI`'s own symbol has no slot to carry" -- a compound `DO` control, still true.
* `ir/golden_tests.rs:331` "A simple variable and a bare stem each resolve to their own slot" -- the **read** side, which already carried the slot at base and is unchanged.
* `plan.rs:103` (`TailPiece::Variable`'s `at`), `plan.rs:167` (`CompoundName::split`), `plan.rs:591` (`note_compound_name`) -- about tail pieces and about a fragment's own split; unaffected.
* `run.rs:2404` and `run.rs:13458` "`ZQXW` appears in no instruction ... so the plan has no slot for it" -- true of a name that appears only inside string literals.
* `run.rs:6922` "the address is a slot and a route" -- a different subject.
* `stem.rs:943`, `error.rs:208`, `trace.rs:118`, `value.rs:151`, `invocation.rs:193`, `ir/mod.rs:1042`, `ir/mod.rs:1176`, `ir/mod.rs:1278`, `ir/compile.rs:2213`, `heap.rs:279`, `rexx-num/lib.rs:429` -- the phrase matched, the subject is not this claim.

## Things I am not sure about, or left undone

* **The bare stem *read* on the tree-walker still hashes its name.** `eval_node`'s `ExprKind::Stem` arm passes `None` to `read_symbol`, and only `Op::Load` carries a slot, so the tree-walker resolves the name at every bare stem read. The entry route this task used would serve both engines there too. **I did not take it**, because `read_symbol` is one of the hottest shared functions in the crate and this task has just measured what perturbing a shared hot path costs; it wants its own measurement on `emptyloop` and `varlookup` first.
* **`stem_drop` from a `DROP` target still resolves by name**, and the slot is reachable: a bare-stem `DROP` target goes through `note_variable_ref` to `note_compound_name`, which records `stem_at`. Wiring it means restructuring `drop_variable`'s `Direct` arm, and `DROP` of a whole stem is not on any hot path. Named here rather than taken.
* **Nothing in the workspace catches the slot not being *used*.** Mutations P3 and P4 -- the two arms ignoring the entry's slot -- leave the suite green, exactly as the previous task's M2 did. That is inherent: taking the slot and resolving the name produce the same answer, so only a measurement can see the difference. The permanent witness is the accessor probe in this report, not a test.
* **The +2-per-pass regression was found by an axis written for a different site.** If the plan had asked only for `stemwrite`, the cost would have shipped: `stemwrite` alone shows -12.6% for the rejected design and says nothing about `emptyloop`. The control axes are what caught it, and they only caught it because their spans were measured first.
* **`rexxcps`' -38.8 million is not decomposed.** Direction and bound only; see above.


## Commits

| commit | what |
|---|---|
| `96d7a87a8` | Take a bare stem's slot from the plan, not from the loop that kept it |
| `866d07d1b` | Record the bare stem's slot as entry 32, appended -- 122 insertions, **0 deletions**, hunk `@@ -3014,3 +3014,125 @@`, a pure append with no earlier entry touched |

**`96d7a87a8`'s parent is `2b7e78450`, not `50faeff88`.** Two commits landed on the branch from elsewhere while this task ran, and **both touch `docs/superpowers/plans/2026-08-13-phase-4g-structural.md` and nothing else** -- `git diff --name-only 50faeff88 2b7e78450` names that one file. So `rust/` is byte-identical at `50faeff88` and at this commit's parent, the base binary every figure was taken with is the right base, and "base `50faeff88`" in the record entry names what was measured rather than the parent in the chain. Recorded here because a reader reproducing the entry from `git log` would otherwise find a base that is not the parent and have to work out why.

---

# Fix round 1

Review: `.superpowers/sdd/2026-08-13-bare-stem-slot/task-1-review.md`.
Everything above this line is left as it was written; corrections to it are below.

## A defect in how the previous round was done, found while fixing it

**Three of the corrections this round was asked to make, I had already made and committed nothing.**
`stem_slot`'s "every `_at` accessor" sentence, `control_slot`'s "the inlining that turns on" clause and `bind_control`'s stem-arm comment were all edited near the end of the previous round, each edit reported as applied, and none of them reached the commit.
The cause is the mutation harness: `mutate.sh` restores every source file from a snapshot directory after each run, and that snapshot was taken **before** those edits.
Its own restore check compares the tree against the same stale snapshot, so it reported OK.
The suite and the gates passed at every point, because only comments differed.

**What makes this worth writing down is that nothing in the round could have caught it.**
The edit tool reported success truthfully; the file was correct when it said so.
The verification that works is to re-read the file for the *new* text after every edit, which is what this round did for each of the fixes below and what caught two further problems in my own replacement prose.

## Findings fixed

### Important 1 -- `assign_expr_target`'s "three different slots"

Corrected.
The sentence is replaced with what the code does: a simple variable and a bare stem write the same slot as each other, a compound writes the stem's, and `at` serves only the first for two different reasons -- it would be the *right* number for a bare stem and was not supplied because supplying it at `bind_control` was measured to cost more than it saves, and it would be the *wrong* number for a compound.
Both of those arms now assert that no caller passed one, so the doc's claim is enforced rather than only stated.

### Important 2 -- the asserted mechanism in `control_slot`'s doc

Corrected, and this is one of the three edits that had been made and lost.
The clause now says what was established: the 2 instructions appear, they are attributed **by partial revert and not by reading the assembly**, undoing the one line puts `emptyloop` back on base exactly, recomputing the slot inside the arm costs +100,000,000, and why the generated code changes was not established.
The "same-binary spans under 1,500" figure, contradicted by this task's own `varlookup` spans, is replaced with the two spans those axes actually showed, 1,348 and 1,576.
The same claim in the plan file's Status block is corrected the same way.

### Important 3 -- established by running, and the reviewer is right

The paragraph said `Plan::note_compound_name` binds a compound's own id to no slot, so a widened `write_slot` would have nothing to answer with.
False: `Plan::bind` puts every name it binds *whole* into `by_symbol`, and `note_loop` and `note_parse` both call it with spellings that may be compound-shaped.

Run, in a debug build, with `write_slot` widened to answer for `ExprKind::Compound`:

| program | `ir` | tree-walker |
|---|---|---|
| `zb = 1; do za.zb = 1 to 2; end; za.zb = 5` | **rc 101**, `a compiled write names a slot this body's plan does not give its target` | rc 0 |
| `zb = 1; parse var za.zb zc; za.zb = 5` | **rc 101**, same | rc 0 |
| `zb = 1; do za.zb over 'one'; end; za.zb = 5` | **rc 101**, same | rc 0 |
| `zb = 1; parse value 'seven' with za.zb; za.zb = 5` | rc 0 | rc 0 |

So the reviewer's claim holds and is narrower than "a compound target": it is a compound spelling that `Plan::bind` reached, which the `DO` control, the `DO ... OVER` control and the `PARSE VAR` source all produce.
A `PARSE` *target* does not, because it reaches the plan through `note_compound_name`.
The tree-walker never panics, because `Op::Store`'s tripwire guards an op it does not read.
The paragraph now says all of that, and says plainly that the green mutation result is a fact about the suite's contents rather than about the code.

**No test of that shape was added, and here is why.**
It would guard against a mutation nobody is proposing, and "can fail" is not "adds coverage".
What the run exposed that *is* worth closing is the tree-walker half: nothing there watches a caller that starts supplying a slot.
So instead of a program, `Interp::assign_expr_target`'s stem and compound arms each gained `debug_assert!(at.is_none())`, which is engine-independent and suite-independent.

**Both new tripwires were shown to fire**, on both engines, under the mutation that is actually plausible -- `parse_template.rs` passing its target's own `by_symbol` slot, which is precisely what this report names as the successor task:

| program | `ir` | tree-walker |
|---|---|---|
| `parse value 'seven' with zs.` | rc 101, `a stem write was handed a slot ...` | rc 101, same |
| `zb = 1; do za.zb = 1 to 2; end; parse value 'seven' with za.zb` | rc 101, `a compound write was handed a slot ...` | rc 101, same |

### Important 4 -- `stem_slot`'s scope, and the coverage gap behind it

Corrected, and the gap closed.

The accessors were enumerated from the file rather than from memory: `read_by_name_at` and `read_stem_at` sit above `stem_slot` and write their own `match at`; `stem_get_at`, `stem_set_at`, `stem_drop_tail_at`, `stem_assign_at` and the `replace_stem` shared with `stem_drop` sit below it and go through it.
The doc now says "every `_at` accessor *below* this one", which is positional and cannot rot into a wrong list.

**The gap was real and closing it was cheap.**
`read_stem_at` now carries a copy of the same tripwire, against the same map.
Whole workspace, `--no-fail-fast`, under `memcap 8G`: **1483 passed, 0 failed, zero firings**.
Inverted to `debug_assert_ne!`: **1474 passed, 9 failed, 8 distinct test names**, including `run::tests::a_stem_control_variable_binds_through_stem_assign` -- the stem-controlled loop, which is exactly the fourth site the review identified as uncovered.
So the read half of the new flow is now checked directly rather than incidentally.

### Important 5 -- `write_slot`'s doc

Corrected.
It no longer says a bare stem goes through `stem_assign` resolving the spelling, which is the path this task removed, and it now states the decision the function exists to make: a bare stem target **does** write the symbol's own slot and the number **is** in `by_symbol`; it is `UNRESOLVED` here because the write reads that same slot off `Code::compound`'s entry, which answers on the tree-walker too, and carrying it here as well would be a second source for one number.

While editing it, the Minor finding about `push_read` turned out to be the same defect: the doc block describing a bare-symbol read was contiguous with `write_slot`'s, so **`write_slot` carried a doc that opened by describing a read**.
The read block is moved down onto `push_read`, where it was written to sit.

### Important 6 -- entry 32's drift paragraph

**Entry 33, appended.**
`git diff --numstat` reads `49 0` and the hunk is `@@ -3136,3 +3136,52 @@`, so no earlier entry is touched.

The arithmetic was redone rather than taken from the review.
Multiples of the larger of each axis's two spans: `varlookup` 24,112x, `strings` 11,079x, `arith` 2,705x, `alloc4c` **51x**, `emptyloop` inside its span.
So "thousands of times their spans" holds for three of the five and not for `alloc4c`.
Fifty times a span is still outside it, so no part of entry 32's disposition moves; the word "thousands" is what is withdrawn.

The `strings` row is the one worth having, and entry 33 says so: this task's own Step 1 base measurement recorded a same-binary span of **36,000,462** on that axis, larger than the -15,000,396 move, and entry 32 carries neither that span nor the caveat.
`strings` therefore carries a bound and no signed figure.
Entry 33 also corrects the spread table's caption: its numbers are the Step 5 spans from the interleaved sitting, not the Step 1 spans measured before anything changed, and it lists the Step 1 set so the discipline the caption claims has its evidence.

## Minor findings

Fixed:

* `debug_assert!(at.is_none())` in `assign_expr_target`'s stem arm, as suggested, and the same in the compound arm, which finding 3's run showed is the half no tripwire watched on the tree-walker.
* `read_stem_at`'s doc named only `Op::Load` as a source; it now names the controlled loop's re-test as the second, and says the two take it from different places.
* "against same-binary spans under 1,500" replaced with the measured spans, in `control_slot`'s doc and in the plan file.
* The fourth site's `867,001,000`: restated in this section rather than left. It is the difference between two variants of the **rejected** design, `10,562,819,715` minus `9,695,819,133`, which is `867,000,582`. The figure above rounds it wrongly and states it beside the shipped design's total. **What is supported is a bound**: on the rejected design's arms, the re-test's read accounts for about 0.87 billion of `stemloop`'s movement. No equivalent partial revert was run against the shipped design, so nothing is claimed about its split.
* Set counts in the plan file: "the three places", "The three sites", "Three comments", "the three applications", "two of the three sites", "the third site" are gone, replaced by the property each was standing in for.
* The fragment test's doc now says it passes unchanged at base and guards the design that was not shipped.
* `push_read`'s missing doc: fixed by moving the block that was already written for it, as above.

Left, with reasons:

* **Entry 32's own set counts** ("Three sites still did it", "Two axes written for this", "Five axes execute no bare-stem operation at all"). The record is appended to, never rewritten, and these are not false. Entry 33 does not restate them.
* **The report body's two-sentence disagreement about `compound`** ("That is codegen drift ... I offer no attribution beyond ..." and "moved -24 million. Drift."). The report above is not edited. **The correction is here: both sentences overreach in the way entry 31 exists to correct.** What the measurement supports for `compound`, `alloc4c`, `arith`, `strings` and `varlookup` is that the movement is **not this change's semantics** -- the accessor probe counts zero bare-stem operations on all but `compound`, which has one -- and nothing beyond that. Neither "drift" nor "layout" is established, because the control that separates them was not built.

## Gates, at the fix round's final state

| gate | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, zero warning lines, `Checking rexx-exec` in the log |
| `memcap 8G cargo test --workspace --no-fail-fast` | exit 0, **1483 passed, 0 failed, 4 ignored** |

No benchmark was re-taken this round, as instructed.
The only executable changes are three `debug_assert!`s, all of which were shown to fire under a mutation and to be silent otherwise.
