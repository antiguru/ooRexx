# Task 10 review: promote the call forms

Reviewed `16077ea1..32d83e9b` on `plan/rust-rewrite`.
Everything below was run in a private copy of the tree under the session scratchpad; the repository working tree was clean at `32d83e9b` before and after.

**Spec compliance: PASS.**
**Quality: good.** The promotion is small, the seam is the one the brief asked for, and the three claims I was asked to attack all survived independent measurement. The findings are documentation, not behaviour.

## Gates, re-run rather than read

* `cargo test --workspace --no-fail-fast` with `REXX_CORPUS_GATE=1 REXX_ASSERTIONS_GATE=1`: **1423 passed, 0 failed**.
* `cargo test --workspace --release --no-fail-fast`, same gates: **1423 passed, 0 failed**.
* `cargo fmt --all --check`: exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings` from a **genuinely fresh** `CARGO_TARGET_DIR`: exit 0, no diagnostic lines.
  My first attempt reused a `cleantarget` directory left over from an earlier session and checked three crates in five seconds; the re-run compiled the workspace and is the one quoted.
* `const _: () = assert!(size_of::<Op>() == 12);` is still at `ir/mod.rs:58` and still compiles, so `Op::Call { index: u32, site: u32 }` did not widen the stream.
* The six new tests in the diff account for the +6 the report claims (four golden, two driver). I did not re-measure BASE.

## What I tried against the cache, and could not break

The strongest instrument was not a program. I patched the hit path in the private copy so that **every** cache hit re-resolves and asserts the fresh answer equals the kept one, then ran the whole workspace with both gates: **1423 passed, 0 failed, no divergence** -- which puts the corpus, the assertion populations and the 10,391-program dual sweep through the check rather than only the programs I could think of.

That instrument is not vacuous. Negative control: making `Calls::remember` store `Resolved::Builtin` unconditionally makes it fire -- `CACHE DIVERGENCE at site 0, name ZSUB`, on a run of one test, not zero.

Hand-built programs, each run three ways (oracle, tree-walker, IR) with stdout, stderr and status kept separate, all three byte-identical unless noted:

* `interpret 'call zsub' zi` in a loop, alternating with a direct compiled `call zsub zi` at a second site.
* `INTERPRET` inside a `::ROUTINE` reaching a label of the routine's own body.
* A `::ROUTINE` and an internal label of the same name, reached from the main body and from a second routine; the same pair through `call zsub` and `call 'ZSUB'` in one loop, which separates the label order from the quoted order at one site pair.
* `SIGNAL` back to a label above the call site, re-entering the same compiled site three times under `trace i` -- full transcript identical to the oracle.
* `SIGNAL` out of a callee; `EXIT` inside a callee; `SIGNAL` out of an **argument** expression's own callee; `EXIT` inside one. These matter for this diff specifically: `32d83e9b` moved the caller's `program`/`program_id`/`selector` reads from above the argument loop to below the builtin return, so an argument that leaves the activation stack changed would be read against the wrong frame. It does not.
* Nested calls inside argument positions, a `>name` reference argument, an argument raising 42.3, an unresolvable name under `SIGNAL ON SYNTAX`, `CALL ON USER` with `RAISE ... RETURN` across the boundary and `sigl` read in the handler, `MAX_ACTIVATION_DEPTH` by infinite recursion (rc 245, both engines identical), a promoted `CALL` as an `IF`/`ELSE`/`WHEN`/`OTHERWISE` body, `TRACE` switched on and off between two calls at different sites, `::REQUIRES` (this crate's declared Phase 5 gap; both engines agree), and `interpret '::routine zfoo'`.

I could not construct a program where the kept answer differs from a fresh one, and I now think the reason is structural rather than lucky: `run_chunk` has exactly one caller (`run.rs:1126`, inside `run_activation`), which looks the chunk up under the running activation's own `body_key`, so a chunk is only ever entered for the body it was compiled from; `Resolved::Label` indexes that body; the builtin table is static; and `Interp::routines` is **append-only** -- `install_directives` refuses a duplicate name with 99.903 (`lib.rs:2124`) -- while a recorded `Label` or `Builtin` outranks any routine anyway. That last step is weaker and better than what the doc claims; see finding 3.

## Findings

* **Minor** -- `crates/rexx-exec/src/clause.rs:208` says "`eval_call` reaches `resolve_and_run_call`". It no longer does: `eval.rs:612` enters `resolve_call` and `invoke_call` directly. The same stale name at `crates/rexx-exec/src/eval.rs:198` names `resolve_and_run_call` as the owner of `>A>`, which after the split is `invoke_call`; `eval.rs:544` cites it for argument evaluation it no longer contains. The task renamed a dozen such references and left others, and the ones naming machinery that moved now point at a two-line wrapper that holds none of it. I opened and adjudicated these three; I did not open every other occurrence and do not claim which of them are false.
* **Minor** -- `crates/rexx-exec/src/ir/mod.rs:960`: "One of the two mutable things a running chunk owns" is a count of an in-repo aggregate in prose, which a third such table falsifies with nothing rereading the sentence. The argument it carries ("an op is emitted once and never rewritten") needs no count.
* **Minor** -- `crates/rexx-exec/src/ir/mod.rs:822`: `CallSite`'s doc rests the cache on `install_directives` "runs once, before the first clause", which nothing asserts and which is stronger than what the cache needs. The property that is actually enforced is append-only-ness (the 99.903 refusal above), and stating that would be both true and stale-proof. Nothing in the suite or corpus violates the stated version today -- `Interp::run` has one call site, `lib.rs:2536`, inside `execute`, which builds its own `Interp`.
* **Design finding, adjudicated** -- the report's reading of the `Sync` interaction is right, and firmer than it puts it. Measured: `assert_sync::<Chunk>()` fails to compile at HEAD naming **exactly one** culprit, `Cell<Option<Resolved>>` in `CallSite` (`ir/mod.rs:834`). "Exactly one" is the load-bearing part -- rustc names no other field, so every other field including `Hints`/`PatchSlot` is `Sync`, and `Chunk` was `Sync` at BASE. Task 9's `AtomicU32` therefore buys nothing a `Cell<u32>` would not, for as long as `CallSite` stays a `Cell`. Two riders for whoever revisits it: the cache hands out `Rc<Chunk>`, and `Rc<T>` is neither `Send` nor `Sync` whatever `T` is, so the bet was already contingent on an `Rc`-to-`Arc` change; and `Resolved` carries two `usize`s plus a tag, so the doc is right that it is too wide for a lock-free atomic here.

No Critical or Important findings.

## The report's own claims, re-measured

Each of these I ran myself rather than read.

| claim | result |
| --- | --- |
| making the shared `>A>` a no-op leaves the sweep green | **green**; the catchers are `both_engines_agree_on_every_case_file`, `call_arguments_covers_...`, `function_call_covers_...`, `run::tests::task_9s_two_new_indents_...` -- exactly the report's row |
| it reddens four rows of `ir_dual_cases/calls` | **four** (rows 6, 8, 10, 18 of 21, measured per row against a pristine binary; seven rows mention `>A>`, and `datadriven` halts at the first failure, so this number is not readable from one run) |
| dropping the `CALL` clause echo op reddens the sweep and names a program | **1 of 10391**, `corpus lang/call_return.rex`, missing `46 *-* call inner` |
| a wrong kept resolution is seen by the case file and the new driver test, and not by the sweep | confirmed: catchers are `both_engines_agree_on_every_case_file` and `a_call_site_resolves_once_and_answers_from_what_it_kept`; the sweep stays **green** |
| no bench axis executes a `CALL` | confirmed: zero clause-initial `call` in all ten `bench-programs/*.rex` |

The `>A>` experiment is also the sharing-rule falsification the brief asked for, and it answers it in the strongest form: **one** edit inside `invoke_call` reddens both a `CALL`-route expectation and a function-route expectation while the engine-against-engine comparison never notices. That is one implementation entered from two routes and two engines, not two copies.

On the brief's "both routes, not one": the implementer's reading is the right one. The brief's own Interfaces line says the two halves are "called by `eval.rs` and the tree-walker **uncached**", which is what was built; the heading sentence is about resolution being shared, not about compiling a native op for `ExprKind::Call`. Their correction 2 should go into the plan.

## Unsettled, and what would settle it

* **A wrong resolution on the hit path has two guards, and the 10,391-program sweep is not one of them.** I reproduced this rather than inheriting it. It is correctly stated as a residual and I would not block on it, but it is the thing to remember when the next cache lands: the sweep's blindness here is structural (both arms are this crate, and almost nothing in those populations reaches one site twice in a way a wrong answer shows), so a second cache gets no free coverage from it. What would settle it permanently is the instrument I used -- the re-resolve-and-compare assertion behind a `cfg` -- which turns every sweep program into a cache check for the cost of one boolean. That is a Task 11+ suggestion, not a change I would ask for here.
* **`CALL_SITE_CACHE = false` is a real switch, not a documented one.** I flipped it and ran the full suite with both gates: 1423 passed, 0 failed. So the "one-line removal" claim holds. Nothing in the tree runs that configuration, so it will rot silently unless something does.
* Two crate-versus-oracle gaps surfaced in probing and neither belongs to this task: `::REQUIRES` is the declared Phase 5 loud gap, and `interpret '::routine zfoo'` reaches the right 99.914 at rc 157 but omits the oracle's first echo line. Both engines agree on both. I did not check whether the second is already recorded.
