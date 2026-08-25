# Task 21, fix round 1 re-review (scoped)

**Verdict: CHANGES REQUIRED.** All eleven findings are genuinely addressed in the tree -- I checked
each against the code rather than against the report, and the two new corpus rows match the oracle
byte for byte on stdout, stderr and exit status on **both** engines. But the C1 fix opened a hole
the report's own soundness argument does not cover, with a three-line witness that was byte-correct
against the oracle at `7d1544f84` and is a loud `rc 120` refusal at HEAD. It is loud rather than
silent, and it is one line to close.

Everything below was run from fresh empty probe directories with absolute paths, three descriptors
read separately, both engines. I did not rebuild.

---

## Per-finding verdicts

| | verdict | what I checked in the tree |
| --- | --- | --- |
| C1 | ADDRESSED | `Activation::context_object` exists, `Interp::context_object` fills it on first ask, the doc no longer claims unobservability. Identity is right for INTERPRET (`1`), an internal `CALL` (`0`), a `SIGNAL ON SYNTAX` handler (`1`), an external `::ROUTINE` (`0`) and a nested class method -- all byte-identical to the oracle on both engines. See NEW-1 below. |
| C2 | ADDRESSED | `unbuilt_collection_owner` asks `directory_scope` first, so it can only ever fire for `.local`/`.environment` (`EnvScope` has exactly those two and `env_seam::which` matches by identity). The check sits **after** `rexx_defined_lock`, so `.Object~defineMethods(.local)` is still `98.985` byte-identical to the oracle. `.methods` still walks and still installs: with a floating `::METHOD`, `.K~defineMethods(.methods)` is byte-identical to the oracle on both engines. D7's sentence is corrected in the tree. |
| M1 | ADDRESSED, and wider than the row pins | The `class_receiver` conversion is gone. I ran four position shapes the corpus does not: `.nil` -> `Class "The K class" has not inherited class "The NIL object".`, `1` -> `"1"`, an uninherited class -> `"The M2 class"`, and the row's own `"abc"`. All four byte-identical to the oracle on stdout, stderr and rc, both engines. |
| M2 | ADDRESSED | `/bin/grep -a "models no removal"` over `crates/` finds nothing. Both replacement reasons are true: `receiver_kind`'s `Body::Stem` arm is `Err("a stem")` (`dispatch.rs:979`), and `a. = 'dflt'; say a.~length` is oracle `4` / crate rc 120 `a message send to a stem is not implemented (Phase 5)`. |
| m1 | ADDRESSED | `ClassClass.cpp:831` is `method_name = stringArgument(method_name, "method name");` and `:832` the `->upper()`; `:961` and `:987` are each the fused call. All three land on code. |
| m2 | ADDRESSED | `Setup.cpp:1218` is `AddMethod("Package", RexxContext::getPackage, 0);`. The added `ContextClass.cpp:160` is `PackageClass *RexxContext::getPackage()`. |
| m3 | ADDRESSED | Both cardinalities struck; the citations still name the right sets (`Setup.cpp:792`-`:804` is nine `RemoveMethod` calls, `:1307`-`:1312` six `HideMethod` calls). The two borderline cases the review listed but did not count are left, as stated. |
| m4 | ADDRESSED | D4's amendment records `hasUninitDefined` with correct citations: `ClassClass.cpp:852` is the `if ((MethodClass *)TheNilObject != methodObject)` and `:854`-`:857` the `strCompare("UNINIT")` / `setHasUninitDefined()` block. |
| m5 | ADDRESSED | Read from the TSV myself: task 20 `arith`/`across_builds`/`pinned>head`/ir/small is `0.989742` in all three of its blocks, task 21 is `0.989743`; ir/large `0.989572` -> `0.989573`. "Five axes, not six" and "Task 20 was not one sitting" are both now what the report says. |
| m6 | ADDRESSED | The new doc is accurate: `RootSet::add_global` (`roots.rs:193`) replaces by name, and `hold_method_object` keys it `method_object_root_key(class, name)`, so a later `~define` under the same name does replace the root. |
| Concern 4 | NOT DONE, and I would not let it stand -- see below | |

---

## NEW-1. The C1 fix's rooting has a third state, and `GC('force')` reaches it

**`Interp::collect_now` is not the only collection site.** `builtin/state.rs:222`, the `GC('Force')`
builtin, calls `interp.heap.collect(&interp.roots)` directly. That path never sweeps `running` and
`suspended` for context objects, and an activation's `context_object` is named by nothing in
`self.roots` while the activation is on the stack. So a forced collection frees a context object the
activation still points at.

Measured at HEAD, both engines identical, oracle `rc 0`:

```rexx
say .context~objectName
say gc('force')
say .context~objectName
```

| | rc | stdout | stderr |
| --- | --- | --- | --- |
| oracle | 0 | `a RexxContext` / `1` / `a RexxContext` | empty |
| ir and tree-walker | 120 | `a RexxContext` / `1` | `rexx-exec: a message send to a value whose object is no longer live is not implemented (Phase 5)` |

**This program was byte-correct at the review's base.** I did not rebuild, so that half is derived
rather than measured: at `7d1544f84` `context_object` was `self.native_instance(class)` with no
cache, so every `.context` minted a live object and both `~objectName` sends answered
`a RexxContext` -- the same three lines the oracle prints. The regression is this round's.

**How bad.** Not memory-unsafe and not a silent wrong answer: handles are generation-checked
(`heap.rs:445`-`:452`), so a stale handle can never resolve to a reallocated slot. Every reachable
consequence is the loud `rc 120` above. It is still a `rc 0` -> `rc 120` divergence on a three-line
program, and no corpus row can see it -- `gc(` appears only in `lang/state_builtins.rex:150` and
`.context` only in four other programs, so the sets do not intersect.

**Two sentences in the tree are falsified by it.**

* `activation.rs:756`-`:760`: "Rooted by `Interp::collect_if_due`'s sweep over the activation stack
  while this activation is running or suspended, and by `Activation::object_roots` while it is
  parked. **Those are the two states an activation can be in and neither reaches the other's
  mechanism.**" The states are two, but the mechanism is attached to the *collection site* rather
  than to the state, and one of the two collection sites has neither. (Separately: the sweep is in
  `Interp::collect_now`; `collect_if_due` only calls it.)
* `dispatch.rs:970`-`:971`, pre-existing: "A handle whose slot is gone. **Not reachable from a
  running program** -- a receiver is rooted by the term that evaluated it." The probe above reaches
  it from a running program.

**Fix.** One line either way. Route the builtin through the sweep -- `interp.collect_now()` in place
of `interp.heap.collect(&interp.roots)`, which also stops a forced collection from leaving
`collect_at` unadjusted -- or make the cache self-checking, `self.heap.get(found).is_some()` guarding
the early return in `context_object`, so a collected entry is re-minted instead of answered. The
first is better: it closes the class rather than the instance, and the sweep is where the design
says the rooting lives.

## The C1 rooting, otherwise: sound, and I witnessed the half the corpus does not

* **Holders of an `Activation` are exactly four**, and I enumerated them from the type rather than
  from the report: `Interp::running`, `Interp::suspended`, `Interp::spare_activations` and
  `DeferredReply::activation` (`activation.rs:916`). The sweep covers the first two, `park_reply`'s
  `anchor` covers the fourth, and the third holds only ended activations whose contents
  `push_activation` overwrites wholesale (`*spare = activation`) before anything reads them.
* **The recycling really does not leak a stale cache.** Three sequential `CALL`s to one internal
  label, each renaming its own context, with 40,000 allocations between them: `a RexxContext` /
  `call1` / `a RexxContext` / `call2` / `a RexxContext` / `call3`, byte-identical to the oracle on
  both engines.
* **The suspended half of the sweep is witnessed.** The corpus row exercises one activation only. I
  ran an outer program that renames its context, calls a class method that renames its own and
  allocates 200,000 strings (far past `COLLECT_FLOOR`, so real collections), then reads both back:
  `inner` / `outer` / `1`, byte-identical to the oracle on both engines.
* **The parked half is witnessed too.** A class method that takes `.context`, renames it, `REPLY`s,
  and reads it back after the main body has run `gc('force')` plus 3,000 allocations: `inner` /
  `inner` / `1` on both engines. (Line order differs from the oracle, which is Phase 6's known
  scheduling divergence and not this round's.)
* **One window exists and is currently empty.** `resume_reply` calls `self.roots.release(parked)`
  before `self.push_activation`, so between those two lines the context object is rooted by neither
  mechanism. Nothing in that window allocates a Rexx object today, so it is latent rather than live.
  Worth a sentence beside the `release`, not a change.

## The perf change (`03756e703`)

**The reasoning holds and the change is safe.** `collect_now` is called from exactly one place, the
body is moved unaltered apart from the new sweep, and `#[inline(never)]` costs one call per
collection. The claim is that the C1 addition inflated `collect_if_due`'s inlined body and so taxed
the *not-taken* path that every allocation runs; the partition is the evidence, and it is the right
kind: the axes that moved are the allocating ones and the three that did not (`compound`,
`emptyloop`, `varlookup`) are identical to six decimal places on both builds.

I checked every figure in the new doc comment against `bench-baselines/phase-5a-arms.tsv` rather
than against the report. All of them read as written: `alloc4c` 1.004808 / 1.010281, `arith`
0.989743 / 0.994651, `strings` 1.013408 / 1.020129, `rexxcps` 1.020337 / 1.026160, and
`dispatchclass` ir/small `[1.017560..1.019626]` against `[1.017565..1.019938]`. The five sittings the
report names are all in the file (`21-fixround-1`, `-1-cold`, `-2`, `-2-inlined`, `-2-committed`).
Keeping the two disk-pressure sittings was the right call for the reason given: they are the control
for the claim that disk pressure did not matter.

## m5 and D4: the report says what the review asked

Both are report-only and both landed, verified against the TSV and against the C++ (above). One new
sixth-decimal slip rode in with them, in the re-measurement section: "**ir/large reads 1.017380 on
both builds**" -- the TSV has 1.017380 out-of-line and 1.017378 inlined. The neighbouring two cells
are quoted correctly as differing pairs. This is the same shape m5 itself was about, one section
later; it changes no conclusion.

## Concern 4: the refusal is defensible in principle and wrong in this instance

The implementer's rule -- do not rewrite another task's measured prose on the strength of one clause
-- is the right rule, and it is the rule this plan has most needed. It does not cover this case, for
two reasons.

**The clause is not merely unwitnessable, it is false, and I measured that rather than reasoning
about it.** `Interp::method_object`'s doc ends "A fresh object per send answers `0` and `a Method`."
On the oracle, two consecutive allocations:

```
a~identityHash   -140155865257009
b~identityHash   -140155865257121
(a~identityHash =  b~identityHash)   1
(a~identityHash == b~identityHash)   0     digits() = 9
```

So a fresh-object-per-send build answers `1` to the `=` row, not `0`. The `a Method` half is sound.

**The remedy asked for is a deletion**, and a deletion cannot introduce a false statement -- which is
the whole of the argument for deferring. Striking `0` and its row leaves the paragraph's other
measurements untouched and un-re-taken. I would take it.

## Citations

Every C++ line this diff adds or changes lands on the code it names. I read each in
`interpreter/`: `ClassClass.cpp:831`-`:832`, `:961`, `:987`, `:1346`, `:1350`, `:852`, `:854`-`:857`;
`Setup.cpp:1218`, `:792`-`:804`, `:1307`-`:1312`; `ContextClass.cpp:160`. `Setup.cpp:1216` is indeed
blank and `:1213` the class-method block, as m2's fix says. This is the first task in five whose new
citations are all on code.

## Comment rules

ASCII throughout -- zero non-ASCII bytes in the added lines, so no em-dashes either. No cardinalities
added. Two things to look at, both minor:

* **Historical framing, `environment.rs:822`-`:823`.** "Measured **before this check existed**:
  `.K~defineMethods(.local)` was rc 0 against the oracle's rc 163." Strike the framing and the
  sentence stops being true of the code as it is, which is the global constraints' own test for
  history that belongs in the report. `Loud::unreadable_collection` two files away makes the same
  point without it ("measured, `.K~defineMethods(.local)` is oracle rc 163, `93.974`").
* **A corpus comment that says something the plain run does not do.**
  `corpus/lang/class_context_identity.rex`: "The loop between the first two rows is deliberate: **it
  allocates enough to run the collector**." It does not, outside the stress harness. `COLLECT_FLOOR`
  is 65,536 slots (`lib.rs:345`) and the trigger needs `slot_capacity() >= collect_at`; 300
  iterations are three orders short. Bracketed empirically by peak RSS on the shipped binary --
  3.4 MB at 300 allocations, 11.8 MB at 40,000, 17.4 MB at 65,000 and flat at 17.4/17.7 MB for
  100,000 and 400,000, so the first collection is at ~65k and the loop is nowhere near it. The next
  sentence, that `collect_stress.rs` is what makes that half real, is true and is the one to keep.

## What I did not check

* I did not rebuild, so the "byte-correct at `7d1544f84`" half of NEW-1 is derived from the diff, not
  measured, and I did not re-run any control.
* I did not run cargo, the gates or `gate_table_c`; the controller's five-gate result and the
  `230 of 230` corpus count stand as reported.
* Scope was the eleven findings. Outside it, one line: `.array~of` is still a Phase 5 refusal, which
  is why my array-position probe for M1 could not run against the crate -- unrelated to this round.
