STATUS: DONE

# Isolating a build outside the shared worktree

A recipe card, written because three agents worked in this worktree at once
today and the isolation step cost each of us time in a different way.

**Why isolate at all.** To measure or mutate without touching a tree other
agents are live in: A/B-ing a fix, mutation-testing your own tests, or
verifying a commit that the working tree has since moved past. Also to be sure
you are measuring what you think, since a working-tree binary can be older than
the commit you mean to test.

## The recipe, for a committed state

```bash
git archive <commit> | tar -x -C "$SCRATCH/iso"
cd "$SCRATCH/iso/rust" && cargo test -p <crate>
```

`git archive` carries the **whole tree** — `interpreter/`, `rust/corpus/`,
everything — so nothing else is needed. No symlinks, no copying files in one at
a time. This is the move whenever the thing you want to test is committed.

## The exception, for uncommitted work

`git archive` only sees committed state, so mutation-testing your own
in-progress change needs a copy instead:

```bash
mkdir -p "$SCRATCH/iso"
(cd "$REPO/rust" && tar --exclude=target -cf - .) | tar -x -C "$SCRATCH/iso/rust"
ln -s "$REPO/interpreter" "$SCRATCH/iso/interpreter"     # in /tmp, never in the repo
```

**A copy of `rust/` alone does not build.** Compilation reaches outside it, and
the failures arrive one at a time, so fixing them reactively costs a full
rebuild per round. The complete list, from a sweep rather than from what I
happened to hit:

| what needs it | path, relative to the repo root |
|---|---|
| `rexx-inventory/build.rs` | `interpreter/messages/rexxmsg.xml` |
| `rexx-inventory/build.rs` | `interpreter/expression/BuiltinFunctions.cpp` |
| `rexx-parse` **lib tests** (`src/directive/tests.rs`) | `interpreter/RexxClasses/CoreClasses.orx` |
| `rexx-parse` **lib tests** (`src/directive/tests.rs`) | `interpreter/RexxClasses/StreamClasses.orx` |
| `rexx-parse` benches (`benches/parse.rs`) | the same two `.orx` files |

Note the third and fourth rows: those are `include_str!` in a `#[cfg(test)] mod
tests` **inside `src/`**, so they are pulled in by `cargo test --lib`, not only
by `--all-targets`. A `--lib` run of a single test still needs them.

**`rust/corpus/` is a compile-time dependency too**, which is easy to miss
because it lives inside `rust/`:

* `rexx-parse/src/instruction/tests.rs` `include_str!`s **13** files from
  `rust/corpus/lang/`,
* `rexx-parse/src/expr/differential.rs` `include_str!`s
  `rust/corpus/expr/precedence.tsv`,
* `rexx-parse/tests/sourceline.rs` `include_bytes!`s
  `rust/corpus/lang/trace_output.rex`.

So copying only `rust/crates/` breaks the build. Copy `rust/` whole.

One consequence worth knowing rather than discovering: because the corpus is
compiled in, an isolated build **pins the corpus to whatever state it copied**.
That is correct for reproducing a commit, and it is a trap if you copy the
working tree while another agent is mid-way through adding corpus files.

## Two traps that are not about paths

**A symlink belongs in the scratch directory, never in the repository.** A
stray `interpreter/interpreter` symlink landed in the tree earlier today.
`$SCRATCH/iso/interpreter -> $REPO/interpreter` is fine; anything under `$REPO`
is not.

**Check the build's exit status without a pipe.** `cargo build … | tail -2` in
an `&&` chain reports `tail`'s status, not cargo's, so a failed build sails
through and the next command measures a **stale binary**. That produced two
wrong stack figures in this phase, by two different agents, on the same day.
Use `${PIPESTATUS[0]}`, or redirect to a log and check `$?`:

```bash
cargo build -p rexx-parse > "$SCRATCH/build.log" 2>&1; echo "EXIT=$?"
```

The symptom is distinctive once you know it: measurements that reproduce the
*previous* code's behaviour exactly.
