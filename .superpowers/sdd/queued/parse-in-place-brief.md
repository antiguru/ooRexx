# PARSE candidate 2: parse the source where it already is

Read `.superpowers/sdd/2026-09-21-parse-cost-scout-report.md` section
"2. Parse the source where it already is, instead of copying it per execution",
and `.superpowers/sdd/2026-09-21-parse-contained-report.txt`, which re-derived
every figure after two commits landed. Those two files are your requirements.

## What the machine stops doing

`rendered_into_parse_buffer`: taking a `Vec<u8>` from the pool, rendering the
source value into it through `to_text`, handing it back, on every execution of
every `PARSE` whose source is a value or an argument. The oracle does not do this
at all.

**400,680,651 Ir, 1.96%**, re-derived at `bb81ae522`. Of the five candidates the
scout ranked, this is **the only one whose instruction count did not move across
two landed commits**, which is why it is next rather than the nominally larger
candidate 1.

Most of it is one inclusive measurement of a call that would stop happening,
`rendered_into_parse_buffer` at 381,360,635 over 2,520,000 calls, 151.3 each,
reproduced to the digit at two revisions. The rest is the pool lines inside
`exec_parse`. **That is a better class of figure than an apportionment**, but it
is still a ceiling: whatever replaces the copy still has to reach the bytes.

## The hazard, which is the whole of the task

The source bytes live in a `Bytes::Inline` inside an arena `Object`
(`rexx-core/src/bytes.rs:41`), and **the template walk allocates**, so the arena
can reallocate and move them mid-walk. A plain `&[u8]` borrow held across the
`&mut self` calls is both unsound and rejected by the borrow checker, so the
compiler will not let you write the naive version. That is fortunate and it is
not a substitute for getting the design right.

The shape the scout proposes: a cursor that holds the **rooted `ObjRef`** and
re-derives the slice after every call that can allocate. **Cheap if done per
trigger, ruinous if done per byte**, because `next_word` indexes the source byte
by byte. Where that line falls is the design question this task exists to answer,
and it is a measurement rather than a judgement.

**`PARSE UPPER` and `PARSE LOWER` must keep copying.** Measured: the upper pass
is 165.2 Ir over the two argument strings, from an A/B that replaced
`parse upper arg` with `parse arg` at the same clause count with `collect_now`
unchanged.

## What a wrong answer looks like here, and what I want built to catch it

A stale slice after a collection gives wrong piece contents, silently. No crash,
no error, and the corpus differential only catches it if a collection happens to
fall inside a template walk during a corpus run.

So: **make a collection happen in the middle of a template walk, show the answer
is still right, and then show that the same test reddens when the slice is not
re-derived.** A test that has never been watched fail is not evidence. This
project has the pattern already for a cache and for a trace guard; this is the
same shape for a borrow.

`tests/collect_stress.rs` is the existing test that exercises allocation during
interpretation and is the natural home or the natural model. **Run the debug
gate for it, not only release**: `Interp::enter_clause`'s `debug_assert` and the
temps-frame watermark are compiled out of everything a `--release` gate runs.

## Ordering, which cost a candidate on the previous task

Candidate 5 measured **slower** on the last task, because candidate 4 had already
taken the three slice indexings candidate 5 was sized against, and the order the
brief happened to specify decided which one banked them.

So: **this task is candidate 2 alone.** If you find a second change worth making,
do not fold it in and report a combined figure. Either measure each against the
same base, or order them deliberately and say in the report why that order and
what the second one's figure would have been at the other order.

## What is settled, so you do not re-derive it

* The variable pattern `(p0)` costs nothing; replacing it with the literal it
  always holds made the run 5,639,869 instructions slower.
* The allocations are required: `alloc_with` is called 2,520,002 times against
  2,520,000 predicted from the template shapes.
* `PARSE` is not where we are losing: 13.71% of our run at `bb81ae522`, against
  21.49% of the oracle's over identical structural work. The remaining gap to the
  oracle's whole implementation is 507,290,565 instructions, and candidates 1 and
  2 together claim more than that, so **at most one of them is worth its figure.**

If one of these is wrong, say so with the probe.

## How it is judged

**The oracle differential is the arbiter.** `corpus_differential` was 604 of 604
in STRICT mode on the last two commits; keep it there.

Gates as `rust/CLAUDE.md` defines them, both pairs, builds outside the cap and
tests under `memcap 8G`:

    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo build --workspace --all-targets --release
    REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release --no-fail-fast
    cargo build --workspace --all-targets
    REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast

Expected: 133 binaries, 2649 passed / 0 failed / 4 ignored release, 2650 / 0 / 4
debug. **`memcap 8G` kills a cold release compile** at rc 137 with `Compiling` as
the last line, which a status-only reading calls a red gate.

A clippy that finishes in under a second against a warm target is provisional;
re-run it from an empty `CARGO_TARGET_DIR` before quoting it. Sum the test
tallies from each log's own `test result:` lines, not from a summary: `cargo
test` stops at the first failing binary, so a summary can describe a run that
never reached the corpus harness.

Commit before any long run and leave the tree frozen until the status file says
finished. **I will not commit anything to this tree while you hold it**, and if
that changes I will tell you rather than let you find it in `git log`.

## Measurement

`valgrind --tool=callgrind` on `rust/bench-rexxcps/rexxcps.rex`, the pinned copy.
Interleaved rounds, own `CARGO_TARGET_DIR` per build, sha256 printed beside each
binary, within-build spread given. BASE at `bb81ae522` is 20,446,440,737.

**Retake the line-level profile afterwards and re-derive candidate 1**, which is
the next decision and which has already been reduced once by work it did not do.
Say what candidate 1 is worth at your HEAD and how much of what remains is cost
that candidate 3 created.

## House rules

`rust/CLAUDE.md` governs. Comments minimal: one-sentence overview, then
parameters, returns, panics and non-obvious properties only. **A comment may
never state the size of a set.** No em-dashes. Never drop or re-wrap an existing
comment; inserting an item under a doc block silently reassigns that doc.

**No `unsafe`.** It is granted only in `rexx-api/src/ffi.rs`, `src/load.rs` and
`rexx-core/src/bytes.rs`, and this task does not touch them. If you conclude the
design needs `unsafe` outside those, stop and report that rather than writing it.

No new dependencies. No process-global state. Never amend a commit.

## Report

Write to `.superpowers/sdd/2026-09-21-parse-in-place-report.md`, or the same path
with `.txt` if your harness refuses `.md`, and say which.

**Your replies to me truncate at about 8 KB. Put gate statuses first**, then
commits, then the measurement, then the re-derived candidate 1, then concerns.
Quote the command beside every figure and give the exit status you observed. If a
run is still going when you report, say so rather than describing what it will
say.
