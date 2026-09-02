## Task 2: the behaviour snapshot (D58)

**Goal.** `corpus/gate-tables/concepts/objcla.rex` agrees, and both arms of D58 have witnesses.

**BASE:** the commit named in your dispatch. Task 5 has landed since this brief was drafted, so
`obdes` now agrees and `class_graph.rs` has moved under it. Read `.superpowers/sdd/2026-08-27-phase-5b/global-constraints.md`
and the 5a constraints it points at, `rust/CLAUDE.md`, the plan's Task 2 section, and D58 in the spec.

**The row is a silent wrong answer, not a refusal.** `objcla.rex` was rc 120 when the plan was
written. Task 1 changed its shape and not in this task's favour, and the controller re-measured this at
`2d5614e63`, where the phase gate still reads `diverge-stdout loud=no 5b objcla`: `Body::Instance` carries its class
and dispatch resolves against that class's instance behaviour live, so the row now reads
`diverge-stdout`, rc 0 on both sides, empty stderr on both sides, `after 1` against the oracle's
`after 0`. That live walk is exactly the mutation `objcla`'s own committed control describes
(`gate_table_c.rs:327`: "rebuild an existing instance's method lookup from its class on every send,
so a method defined after the instance was created answers"). Closing it is this task's whole
subject. **Re-measure this before you build**; it is Task 1's report's claim, not a premise.

**The rule (D58).** An instance dispatches against the behaviour its class held at construction, and
the two families of class mutator differ:

* `~define`, `~defineMethods`, `~delete` replace the class's instance behaviour with a **copy**, so
  existing instances keep the old one -- `ClassClass.cpp:860`-`:862`, `:531`-`:533`, `:962`-`:964`,
  each with the C++'s own comment saying so;
* `~inherit`, `~uninherit` rebuild the existing behaviour **in place** (`:1361`, `:1413`, both into
  `updateSubClasses` at `:1036`), so existing instances **do** see the change.

**Make it a property of where the behaviour lives, not a rule restated at each mutator.** D58 says
this outright and gives the reason: a live walk of the class graph on every send gets `~inherit`
right and `~define` wrong; a copy on both families gets `~define` right and `~inherit` wrong. Both
are rc 0 with empty stderr, so neither is loud and no existing gate row separates them.

**What is already there, re-measured by the controller at `2d5614e63` -- re-measure it yourself, it
is a claim and not a premise.** `rexx_classes::ClassGraph` already models both families and exposes
the handle family this task needs, all in `crates/rexx-classes/src/class_graph.rs`: `pub struct
BehaviourHandle(usize)`, `instance_behaviour_handle`, `has_method_at`, `lookup_at`,
`resolve_super_scope_at`. Cited by symbol and not by line on purpose: an earlier draft of this brief
carried line numbers and four of the five had rotted by the time the task was ready to dispatch,
because Task 5 added to the same file. Find them with `grep -n`. What blocks putting a handle in the body is a real dependency cycle and not
a style rule: `rexx-classes/Cargo.toml` depends on `rexx-core`, and `rexx-core` depends only on
`rexx-num`, so `Body` naming a `rexx-classes` type would close a loop. `BehaviourHandle` is a newtype
over `usize`. Deciding what moves where is yours; **prefer deleting to rewriting**, and say in the
report what you chose and what you rejected.

**Done when**

* `objcla` agrees on both engines under
  `REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast`,
  and its probe path is in `corpus/phase-5b.txt` in the same commit;
* **`corpus/phase-5b.txt` carries the `~inherit` and `~uninherit` witnesses.** Neither arm exists in
  the corpus today, and the load-bearing half of that is not how many files mention inheritance but
  that **none of them constructs an instance**: from `rust/corpus`,
  `git ls-files '*.rex' | xargs /bin/grep -ail '~ *inherit\|~ *uninherit'` names the files that
  mention either, and piping those through `xargs /bin/grep -ail '~ *new'` names none of them.
  Re-run both; the first set grew during Task 5 and will grow again. The spec prints the `~inherit`
  program; the `~uninherit` arm it states in prose only, so it is written out here:
  `k = .Object~subclass('K')`, `k~inherit(.Mx)`, `o = k~new`, then `k~uninherit(.Mx)` gives
  `a 1` / `b mixin-ran` / `c 0` / `e trapped 97` at oracle rc 0, with `.Mx` a `MIXINCLASS Object`
  carrying `mxm`. **Measure it yourself against the oracle; do not copy those four lines on trust.**
* **two controls are recorded as run**, each with its transcript:
  1. `objcla`'s own committed control -- rebuild the lookup from the class on every send -- reddens
     `objcla`. (This is the pre-task state, so it is the cheapest of the two and proves the row is
     live rather than green over nothing.)
  2. **copying the behaviour on `~inherit` too reddens the new `~inherit` witness, while every gate
     row that agreed before the mutation still agrees after it.** This is the point of the task: the
     mutation no existing row can see. Phrase it against the rows that were green before the
     mutation, not against "every gate row" -- several 5b rows are still red for reasons that have
     nothing to do with it. Record which rows those were, by name, in both readings.

**Hazards this phase keeps hitting, named so you can defeat them rather than rediscover them.**

* **"Can fail" is not "adds coverage."** Before claiming a new witness earns its place, run the
  mutation against the suite *without* the new program. If an existing row already catches it, say
  so and say what the new one adds that nothing else does.
* **A witness that cannot fail.** Task 1 found two: a program whose instance is held in a caller
  variable roots itself, and a five-byte exposed value never becomes a heap object. Check that your
  witness's own value can move.
* **Probe past the row.** Every 5b task turns a loud refusal into an answer, which is when a silent
  wrong answer gets introduced. This one changes *the send*, which every program in the corpus uses.
  Look for shapes the snapshot rule changes that no row covers: `~define` on a superclass after
  construction, `~inherit` on a superclass, an instance built before and after the same mutation,
  `~setMethod`'s interaction if any, and a subclass's instances under a parent's mutation.
* **No comment states the size of a set.** ASCII only, no em-dashes, no historical framing.

**Then run all five gates**, from `rust/`, each status read unpiped, plus the phase-gate command
above. Write your report file first and append as you go; do not poll long commands on a short
interval; message the controller when you finish.
