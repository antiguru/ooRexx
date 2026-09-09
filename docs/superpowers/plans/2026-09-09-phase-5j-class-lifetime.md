# Phase 5j — class lifetime, implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to
> implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Collect user-defined classes like ordinary objects, keeping built-in classes static, so
that `~subclasses`, `WeakReference` and resident set all match the oracle.

**Architecture:** The class registry stops being a reason for a class to be live. `Heap` gains a
mark bit per class identity, set when tracing reaches a class handle; a marked class's payload is
traced through an edge supplier the caller passes in; and between the `UNINIT` pass and the next
registry read, unmarked non-built-in classes are expunged. Class ids are never reused. Built-ins are
marked every cycle rather than skipped.

**Tech Stack:** Rust, `rexx-core` (`heap.rs`, `handle.rs`, `body.rs`), `rexx-classes`
(`registry.rs`, `class_graph.rs`), `rexx-exec` (`lib.rs`, `run.rs`, `dispatch.rs`).

**Spec:** `docs/superpowers/specs/2026-09-09-phase-5j-class-lifetime.md`

## Global Constraints

* **No `unsafe`.** The workspace lint is `deny`; a per-site exception is Moritz's alone to grant and
  has not been granted for this phase.
* **Correctness is byte-identical stdout, stderr and exit status against the C++ oracle**, read on
  **three separate descriptors, never `2>&1`**. Oracle runs use the standard wrapper from a fresh
  empty directory.
* **Both engines.** Every behavioural witness runs under `Engine::TreeWalker` and `Engine::Ir`.
* **Four gates, all green at the closing commit:** `cargo fmt --all --check`; `cargo clippy
  --workspace --all-targets -- -D warnings`; `cargo test --release --workspace --no-fail-fast`;
  `REXX_CORPUS_GATE=1 cargo test --workspace --no-fail-fast`. **Commit before a long gate run and
  leave the tree frozen until it finishes.**
* **Comments are minimal**: a one-sentence overview, parameters/returns/panics, and non-obvious
  properties only. Measurements, alternatives and reasoning go in the spec or a record, not in a
  comment.
* **Every extent claim is derived and its command committed.** No "all", "none", "every" or a count
  in prose without the enumeration that produced it.
* **Write each negative control's prediction before running it**, and mark each part confirmed,
  falsified, or unobservable.
* Never `git add -A`; never amend; never force-push; never `git reset --hard`; never
  `git checkout -- <path>` on an edited file; never bare `git stash` / `git stash pop`.

---

### Task 1: A `WeakReference` to a live class stops reading `.NIL`

Independent of everything else in the phase, and a wrong answer today. Lands first so the rest is
built on a correct weak protocol.

**Files:**
- Modify: `rust/crates/rexx-core/src/heap.rs` (the weak-clearing pass, around line 196)
- Create: `rust/corpus/lang/weakref_class.rex`
- Modify: whichever `rust/corpus/phase-*.txt` list the corpus harness reads for `lang/` programs

**Interfaces:**
- Consumes: nothing.
- Produces: nothing. The predicate changes again in Task 4; that is expected and noted in the code.

- [ ] **Step 1: Write the failing witness**

`rust/corpus/lang/weakref_class.rex`:

```rexx
/* A WeakReference to a class answers the class while the class is live.
   The ordinary-object case is the control: it is correct today, and it is
   what says the class case is about classes and not about weak references. */
c = .Object~subclass('TEMPC')
w = .WeakReference~new(c)
call gc 'force'
v = w~value
if v == .nil then say 'live class: NIL'
else say 'live class:' v~id
o = .Object~new
w2 = .WeakReference~new(o)
call gc 'force'
v2 = w2~value
if v2 == .nil then say 'live object: NIL'
else say 'live object:' v2~objectName
w3 = .WeakReference~new(.DECL)
call gc 'force'
if w3~value == .nil then say 'declared: NIL'
else say 'declared:' w3~value~id
::class DECL
```

- [ ] **Step 2: Run it against the oracle and record the expected bytes**

From a fresh empty directory:

```sh
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  /home/moritz/dev/repos/ooRexx/build/bin/rexx weakref_class.rex )
```

Expected, measured 2026-09-09: `live class: TEMPC`, `live object: an Object`, `declared: DECL`,
rc 0, empty stderr.

- [ ] **Step 3: Run the crate and confirm it diverges**

Expected before the fix: `live class: NIL` and `declared: NIL`, with the control line correct.

- [ ] **Step 4: Widen the predicate**

In `heap.rs`, the weak-clearing pass currently reads:

```rust
let target_alive = self.resolve(target).is_some_and(|t| self.marks[t]);
```

Change it to:

```rust
// A class identity is not an arena handle, so `resolve` answers `None` for
// one and the rule above would read every class as dead. Nothing collects a
// class yet, so every class is live; Task 4 replaces this term with the
// class mark.
let target_alive =
    target.class_id().is_some() || self.resolve(target).is_some_and(|t| self.marks[t]);
```

- [ ] **Step 5: Confirm the witness now agrees with the oracle on all three descriptors, both engines**

- [ ] **Step 6: Negative control, prediction first**

Write the prediction down, then run: with the `target.class_id().is_some() ||` term removed,
`weakref_class.rex` reddens on both engines and no other test does. Record which parts were
confirmed, falsified, or unobservable.

- [ ] **Step 7: Commit**

---

### Task 2: A class is an ordinary arena object

**Supersedes the reverted Task 2** (`8080cb981`, reverted `bf91af935`), which kept classes outside
the arena and built a mark store, an edge supplier and a built-in seed around that. Spec §4.0 has
what changed and why.

**Files:**
- Modify: `rust/crates/rexx-core/src/handle.rs` (delete `CLASS_SLOT_BASE`, `ObjRef::class`, `ObjRef::class_id`, `is_class_slot`)
- Modify: `rust/crates/rexx-core/src/body.rs` (the new `Body` variant and its `trace` arm)
- Modify: `rust/crates/rexx-classes/src/native_classes.rs` (take a minting callback)
- Modify: `rust/crates/rexx-exec/src/lib.rs`, `dispatch.rs`, `eval.rs`, `run.rs` (the discriminator sites)

**Interfaces:**
- Produces: a class identity that is an ordinary handle with a slot and a generation; the
  class/non-class test as a `Body` match arm.

- [ ] **Step 1: The `Body` variant**

A class body holding what the class owns, traced like any other body. It holds no `rexx-classes`
type — the registry keeps the class *structure* and is keyed by identity, which D76 leaves alone.

- [ ] **Step 2: Mint through a callback**

`native_classes::build` takes `&mut dyn FnMut() -> ObjRef`. `rexx-classes` still does not touch the
heap; `reserve_id`'s counter goes.

- [ ] **Step 3: Built-ins are allocated immortal**

`Heap::alloc_immortal`, which is already a traced root — `collect` chains `immortal` into the work
list. Assert the count is what §1 measured (68) rather than trusting the wiring, because an empty or
short immortal set is the silent no-op §6 predicts.

- [ ] **Step 4: The discriminator sites**

Each of the sites named above already fetches the object on its fall-through, so each becomes a
`Body` arm. Do them by following compiler errors from the deletions in Step 1, not by grep — the
deletions make the compiler enumerate the set.

- [ ] **Step 5: Task 1's predicate returns to one term**

Delete the `target.class_id().is_some() ||` term. **`weakref_class.rex` must still pass**, and that
is the assertion that the revert did not reintroduce the defect it fixed.

- [ ] **Step 6: Witness, negative controls, gates, commit**

Predictions written first. At minimum: with built-ins not immortal, name which test reddens; with
the new `Body` arm not traced, name which.

---

### Task 3: What a class owns moves into the class object

**Three tables, not one** — `class_variables`, `method_objects` and `annotations` are all per-class
arena objects held by a named global root, and `roots.rs` has no `remove_global`, so they cannot be
unrooted. They become fields of the class body, reached by tracing.

- [ ] **Step 1: Delete the per-class `add_global`s**

`run.rs:3207`'s `add_global(&format!(".class-variables {class}"), owner)` and the method-object and
annotation roots. Worth seeing while doing it: `globals` is `Vec<(String, ObjRef)>` and `add_global`
linearly scans it comparing strings, so a per-class root is not only permanent but an entry in a
vector a class-creating program grows without bound. No figure is claimed; the shape is in the type.

- [ ] **Step 2: Witness the pool survives a collection**

A class variable wide enough to need a heap slot, read back after a forced collection, on both
engines and under `run_program_collect_every_alloc`.

- [ ] **Step 3: Negative control, prediction first**

With the class body's trace arm returning nothing, the witness reddens under the stress mode.
Predict whether the ordinary run reddens too, then measure — the answer says whether the collection
is the only thing that sees it.

- [ ] **Step 4: Full gates, then commit**

---

### Task 4: The sweep unlinks the registry

The first task with an observable behaviour change. **Not a separate expunge pass** — the sweeper
already frees slots and bumps generations; this is the registry row going with the slot, which is
Ruby's shape and O(1) per dead class.

**Files:**
- Modify: `rust/crates/rexx-core/src/heap.rs` (report dead class ids in `Stats`)
- Modify: `rust/crates/rexx-classes/src/registry.rs`, `class_graph.rs` (row removal)
- Modify: `rust/crates/rexx-exec/src/lib.rs` (`collect_now` applies the expunge)

- [ ] **Step 1: Report what the sweep freed**

The sweeper already frees unmarked slots and bumps their generations. A freed slot whose body was a
class body is reported in `Stats`. Nothing is special-cased for `UNINIT`: a class with a pending
finalizer is resurrected by the machinery that resurrects any other object, so it is not freed and
therefore not reported.

- [ ] **Step 2: Remove the rows**

`ClassRegistry::expunge(&mut self, dead: &[ObjRef])` removes the class from `graph`, `names`,
`default_names`, `object_names`, `by_name` and `by_system_name`. The list of maps is derived from
the struct definition with an exhaustive destructure, not from memory.

- [ ] **Step 3: A recycled slot is a miss, not an alias**

Assert the property the generation buys: define a class, collect it, define another, and check that
a handle to the first misses rather than answering the second. This is the test that would have had
to be a never-recycle rule under the reverted design.

- [ ] **Step 5: Witness**

`corpus/lang/class_collected.rex`: create, drop, force, and read `.Object~subclasses~items` back to
its baseline; plus the weak reference clearing after the drop. Oracle-checked on three descriptors,
both engines.

- [ ] **Step 6: Negative control, prediction first**

With the expunge suppressed, the witness reddens. With built-ins wrongly included in the dead set,
predict which test reddens first, then measure.

- [ ] **Step 7: Full gates, then commit**

---

### Task 5: Subclass lists hold weak entries and are scrubbed

**The task that decides whether the phase does anything.** A strong `subclasses` vector on `.Object`
pins every runtime class forever, and every gate stays green over it.

**Files:**
- Modify: `rust/crates/rexx-classes/src/class_graph.rs` (`ClassDef::subclasses`)
- Modify: `rust/crates/rexx-exec/src/dispatch.rs` (`~subclasses`)

- [ ] **Step 1: Stop the list being a mark source**

`subclasses` must not be reachable from Task 2's `payload`. Assert it rather than assume it: a test
in which the only path to a class is its superclass's subclass list, and the class is collected.

- [ ] **Step 2: Scrub at expunge**

Every surviving class's `subclasses` loses the dead ids, in Task 4's expunge. The oracle prunes as
it reads (`getSubClasses` → `weakReferenceArray()`); pruning at expunge is the same observable and
costs nothing per read.

- [ ] **Step 3: The behaviour cascade skips dead entries**

`class_graph.rs:582` and `:596` walk `subclasses`. After Step 2 they cannot see a dead id; assert
that with a test rather than relying on ordering.

- [ ] **Step 4: Witness, negative control, gates, commit**

---

### Task 6: A send to a collected class is loud

- [ ] **Step 1:** A handle to an expunged class reaching a send answers a named refusal at rc 120,
  never a panic and never a silent wrong answer. Witness with a program that keeps a class handle in
  a variable the collector cannot see — if no such program can be written, say so and record why,
  because that is a stronger result than a test.
- [ ] **Step 2:** Gates, commit.

---

### Task 7: Close the phase

- [ ] **Step 1: The derived enumeration**

Commit the artifact listing every site that holds a class identity strongly, with the command that
produced it. `Interp::object_roots`' class-identity comments are the starting set and are updated to
match.

- [ ] **Step 2: The growth witness**

A class-creation loop's resident set is bounded. Measure the subject directly — a wrapper's rusage
measures the wrapper, not the interpreter.

- [ ] **Step 3: D59a's four consequences**

Each gets a committed witness or an explicit statement that it could not be witnessed:
`UNINIT` on a dropped class, `~subclasses` counting one, a `WeakReference` answering one, and
growth under class creation.

- [ ] **Step 4: The gate document**

`docs/superpowers/plans/phase-5j-gate.md`, assessed and recorded, in the form the 4a–4e gates use.
Every criterion in the spec's §7, each with the command that produced its reading.

- [ ] **Step 5: Update the 5b spec**

D59, D59a and D60 are marked resolved by this phase, with the commit that did it. The measurements
in D59 stay; the licence does not.
