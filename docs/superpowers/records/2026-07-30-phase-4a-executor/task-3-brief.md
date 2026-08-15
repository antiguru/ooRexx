### Task 3: The borrow-shape spike

**Spec:** "Architecture / The borrow shape", and D19.

**Files:**
- Create: `rust/crates/rexx-exec/Cargo.toml`, `src/lib.rs`, `src/bin/rexx-run.rs`
- Create: `rust/crates/rexx-exec/tests/spike.rs`
- Modify: `rust/Cargo.toml` (workspace members)

**Interfaces:**
- Produces: `Interp` owning `Heap`, `RootSet`, `Vec<Activation>`, `plans: HashMap<BodyKey, Rc<Plan>>`, an output sink and a trace sink; `pub fn run_program(text: Vec<u8>) -> Outcome`, executed on a dedicated thread.

**Why:** this is the phase's one unsolved architectural question and the parent plan says to spike it first. The deliverable is a proof that compiles, not a design note.

- [ ] **Step 1: Prove the shape with the smallest possible interpreter**

Enough of `Interp` to execute `say 'hello'` and nothing else. The discipline being proven:

```rust
// The instruction loop clones the Rc into a local on entry, and every
// &CodeBody and &Expr derives from that local. The activation's own Rc is a
// liveness anchor and is never borrowed through.
let program = Rc::clone(&self.activations.last().expect("a frame").program);
let body = &program.main;
while let Some(instruction) = body.instructions.get(self.pc()) {
    // self.eval(...) takes &mut self, which only compiles because `body`
    // borrows `program`, a local, rather than borrowing self.
    self.step(body, instruction)?;
}
```

Put the version that does **not** compile in a comment beside it, with its `E0502`, because the next phase to touch this will want to know which shape is wrong:

```rust
// Does not compile: borrows self, then calls &mut self.
//   let body = &self.activations.last().unwrap().program.main;
//   self.step(body, ...);           // E0502
```

- [ ] **Step 2: Prove it survives a fragment created mid-instruction**

A test that parses a fragment at run time with `parse_interpret`, executes its body inside the current activation, and returns. The fragment's `Rc` is a local that outlives the nested loop. This is 4a building the machinery; the `INTERPRET` *instruction* is 4b's and still fails loudly.

- [ ] **Step 3: Run the interpreter on its own thread**

**The sized thread belongs to `rexx-exec`'s public entry point, not to the `rexx-run` binary**, because the L0 harness and the assertion-table harness both run in process and a `cargo test` thread's default stack is far smaller than the one the depth limit is calibrated against. Put it in the binary only, and every in-process caller sits on the cliff the depth policy exists to keep them off.

That entry point spawns a thread with an explicit stack size, and **that thread owns everything from `parse_program` onward** — bytes in, an outcome out. `Rc<Program>` is `!Send`, so a program parsed on the main thread cannot be handed across, and getting this wrong is a compile error on day one rather than a subtle bug.

Record the chosen stack size and the measured per-frame cost in the task report; Task 11 sets the depth limit from them.

- [ ] **Step 4: Verify**

Run: `cd rust && cargo test -p rexx-exec && cargo clippy -p rexx-exec --all-targets -- -D warnings`
Expected: the spike tests pass, `say 'hello'` prints `hello`.

- [ ] **Step 5: Commit**

```bash
git add rust/crates/rexx-exec rust/Cargo.toml
git commit -m "Spike the executor's borrow shape: Interp owns everything but the AST"
```

---

