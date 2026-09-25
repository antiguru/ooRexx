# Gate table C repair: review

Reviewed `a442e0b5b..5757857a4` (commits `a4b6a37ce`, `5757857a4`). Read-only on the
worktree throughout; all builds ran against a `git archive HEAD` extraction under this
session's scratchpad with its own `CARGO_TARGET_DIR`.

## Verdict: CHANGES REQUESTED

The green is real for File, Stream, Alarm and Ticker, and for 7 of StreamSupplier's 8
rows' *logic* -- but StreamSupplier's committed construction expression is a hardcoded
absolute path into this one developer's home directory
(`/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/corpus/gate-tables/fixtures/streamsupplier_seed.txt`),
not a path resolved relative to wherever the tree is checked out. It answers correctly
here only because this worktree happens to sit at that exact path. Confirmed: a
`git archive` extraction to a different absolute path still carries the identical
literal string, pointing back at this worktree rather than at its own copy of the
fixture -- `rust/crates/rexx-extract/src/docs/classes.rs:219` and the committed
`streamsupplier__instance.rex`. This project routinely builds dedicated `-gates`
worktrees at pinned commits for gate verification (`git worktree list` shows
`ooRexx-gates`, `ooRexx-gates-b`, `ooRexx-5i-gates` today) -- the next such worktree
built at or after `a4b6a37ce` will not see this file and the row will regress, most
likely back to `unanswered` (silent) or to a genuine spurious divergence if the two
engines' file-not-found paths differ even slightly, either of which reddens or
silently un-fixes exactly the gate this repair exists to close. `gate_table_c.rs`'s own
`corpus_dir()` (line 30) and `method_bodies.rs`'s override table both already resolve
paths via `env!("CARGO_MANIFEST_DIR")` or OS-global paths (`/`, `/dev/null`,
`/etc/hostname`) rather than a baked-in absolute repo path, so a portable pattern was
available and not used here. The brief's own guidance ("prefer a receiver whose
construction cannot depend on the filesystem outside a directory you create") is not
met by a receiver that depends on one specific filesystem location outside any
directory this run creates.

## Per-item findings

1. **Construction expressions.** File (`.File~new('/nonexistent-gate-table-c-file')`) and
   Stream (`.Stream~new('/nonexistent-gate-table-c-stream')`) independently verified
   against the oracle from a fresh directory: both construct without raising and answer
   real `hasMethod` true/false values, not a degenerate or trivial receiver. StreamSupplier
   also verified against the oracle (from this worktree, where its absolute path
   resolves) and answers correctly, but the construction itself is non-portable -- see
   verdict above. Alarm and Ticker's cancelling-cascade construction is real and
   matches the oracle's documented route (`.Alarm~new(...)~~cancel`), verified below.
2. **The 82 rows.** Re-ran File, Stream and StreamSupplier's committed instance probes
   directly (the worktree's own `target/debug/rexx-run`, interpreter logic untouched by
   this diff, against the oracle, three descriptors): all three exit 0 with byte-identical,
   non-empty `instance 1`/`instance 0` stdout on both sides -- real answers, not both
   sides refusing identically. `Verdict::Agree` requires all three channels (status,
   stdout, stderr) to match byte for byte (`gate_tables/mod.rs:89`-`100`), so a shared
   refusal would show as `Agree` too in principle, but that is not what happened here:
   the sampled rows carry real boolean answers.
3. **Regenerated row sets.** Symlinked the real `oodocs/` into the pristine extraction
   and re-ran `cargo run -p rexx-extract --bin rexx-extract-docs -- --oodocs ../oodocs
   --interpreter ../interpreter --out <dir>` from `rust/`: all five output files,
   including `class-set.txt` and `class-methods.txt`, are byte-identical to what is
   committed. No row's `status` moved in a direction that excuses a failure --
   every changed row moved `not-covered` to `covered` (more surface exposed to gating,
   not less), confirmed by diffing every File/Stream/StreamSupplier/Alarm/Ticker row.
4. **`method_bodies.rs` / `method-bodies.txt`.** Forced consequence, not an independent
   edit: `instance_receiver` (`method_bodies.rs:425`) falls back to `class.construction`
   whenever a class has no `RECEIVER_OVERRIDES` entry, and Alarm/Ticker are absent from
   that table (confirmed by reading it in full), so class-set.txt gaining their
   construction automatically changed what this table sends. `check_receivers_match_table_c`'s
   skip for File/Stream/StreamSupplier still has a live reason: those three keep
   deliberately different, working receivers in `RECEIVER_OVERRIDES`
   (`/`, `/dev/null`, `/etc/hostname`) because this table sends real documented methods
   rather than only `hasMethod`, and the new doc comment at `method_bodies.rs:621`-`627`
   states that correctly.
5. **The 13 loud rows.** `phase-4-exclusions.txt`'s new entry records Alarm/Ticker as an
   OWNER: Phase 6 exclusion with a transcript, not a licensed divergence. Reproduced
   independently: crate exits 120 with `"alarm_startTimer"`/`"ticker_createTimer" ...
   not implemented (Phase 6)` (plus the GUARD-wait line for Alarm) against the oracle's
   rc 0 and seven/six `instance 1` lines -- matches the exclusions entry's transcript
   exactly.
6. **`phase-7-gate.md` section 9.** Added as a new dated section after section 8's own
   text, which is untouched (diff is pure addition). Its claims check out: `98a0db498`
   (2026-09-12) is where `"7"` entered `CLOSED_PHASES`; the repair commit is `a4b6a37ce`
   (2026-09-16); four days between them.
7. **Prose.** One inaccuracy tied to the finding above: `gate-tables/README.md`'s new
   line ("committed here rather than pointed at a path outside the repository") and
   `method_bodies.rs`'s new comment ("reads a fixture this crate's corpus carries") are
   both literally true (the fixture *file* lives under `corpus/`) but invite the false
   inference that the *reference* to it is repo-relative and portable; it is not. No
   comment-cardinality or unfalsifiable "every X" issues found elsewhere in the diff
   (the two "every"/"only" hits that looked live turned out to be either a quotation of
   section 8's own wording or a checkable claim about gate table C's fixed,
   `check_probe_text`-enforced probe shape).

## Answer to the review question

The green is not vacuous and not a changed question in the sense the brief warned
about (no degenerate receiver, no `hasMethod`-answers-trivially trap, no relaxed
verdict, no gated-to-ungated status move) -- File, Stream, Alarm and Ticker's rows are
real. But StreamSupplier's row is green today only because of where this specific
worktree happens to live on disk, which is a different but related failure mode: the
gate's reproducibility, not its verdict logic, is what silently changed. Fix by
resolving the fixture path the same way `corpus_dir()` already does (`env!("CARGO_MANIFEST_DIR")`-relative), then re-verify the row and re-derive the three files
`CONSTRUCTION_PROGRAMS` feeds.

No other findings.
