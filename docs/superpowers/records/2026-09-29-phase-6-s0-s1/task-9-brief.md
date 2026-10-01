### Task 9: Sends, argument positions and the other resumable entries

**Files:** `rust/crates/rexx-exec/src/ir.rs` (`Op` enum, `NodePath` `:309-319`), `ir/compile.rs`
(`native_shape` `:1212-1235`, call arguments `:914-918`, function-call arguments `:1417`),
`ir/drive.rs` (`Op::EvalExpr` `:864`, `Op::Message` `:1619`), `dispatch.rs` (`send_message` `:2569`,
`invoke` `:1907`, `enter_method_body` `:2152`, `resume_reply` `:2492`), `dispatch/object_protocol.rs`
(`started_message` `:824`, `dispatch_held_message` `:893`), `run.rs` (`exec_message` `:1154`),
`dispatch/executable.rs` (`Routine~call`).

**Interfaces:** Produces `Op::Send` appended after the last variant (never before `Op::Clause`), with
`size_of::<Op>() == 16` kept; per-site node addressing for expressions in argument position; the
begin and finish halves of a method invocation.

- [ ] **Step 1:** Witnesses first: a collect-stress test allocating inside a method while the sender
      holds a value; corpus programs for a send inside a function argument, `.message~new(o,'M')~send`,
      `obj~start('M')~result`, a REPLY whose continuation prints, `Routine~call` of a Rexx routine,
      and `~new` whose INIT sends; each identical on the oracle. A test that a guarded method entered
      by `Op::Send` still reports ISGUARDED as today.
- [ ] **Step 2:** `native_shape` accepts `Message` (no `super_class`, no cascade), `DotVariable` and
      `List` leaves, and calls and sends in argument position, with per-site addressing replacing
      `path: None` (`NodePath` is a binary path with a width limit; extend it or add a per-site table,
      and record which in the report). No `Op::Generic`.
- [ ] **Step 3:** `Op::Send` and `Op::Message` enter a Rexx method stackless; `~new` into INIT,
      `Message~send`/`~sendWith`, `Object~send`/`~sendWith`, `Routine~call` of a Rexx routine, a started
      message's first send and the REPLY resume become resumable entries (today they still run
      synchronously or deferred; only their Rust recursion goes).
- [ ] **Step 4:** Corpus, collect-stress, the REPLY and Message corpus programs, and the structural
      tests green. Callgrind; the send loop is the program the spike lost most on.
- [ ] **Step 5:** Gates; commit.

