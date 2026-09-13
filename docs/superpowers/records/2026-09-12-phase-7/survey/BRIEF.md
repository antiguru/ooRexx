# Phase 7 survey — shared brief for every surveyor

## Context

A clean-room Rust reimplementation of ooRexx 5.3 lives in
`/home/moritz/dev/repos/ooRexx-rust-rewrite/rust` (branch `plan/rust-rewrite`, HEAD `5bcb28edb`).
Correctness is **differential**: byte-for-byte agreement with the C++ oracle on stdout, stderr and
exit status, read as three separate descriptors. Phases 0-5 are closed.

**Phase 7 is "Streams & platform"**: roadmap row at
`docs/superpowers/plans/2026-07-27-rust-rewrite.md:476`. Read these before starting:

- `docs/superpowers/specs/2026-09-04-phase-7-scoping.md` — the earlier scoping survey, and its
  "Decisions taken" section at the end.
- `docs/superpowers/plans/2026-07-27-rust-rewrite.md:299-346` — D11 (RexxUtil) and D12 (security
  manager).
- `docs/superpowers/plans/phase-4-exclusions.txt` — grep it for "Phase 7"; every row there is owed.
- `rust/CLAUDE.md` — its "The oracle" and "Probes" sections are binding on you.

You are one of six parallel **read-only** surveyors. Your output is **one report file**. The Phase 7
spec will be written from the six reports. You write no code.

## Scope rulings already made (provisional — flag evidence against them, do not re-argue them)

- `RexxQueue`, the external-queue entry points (`rexx_*_queue`) and `.STDQUE` → **Phase 10**
  (RXAPI-backed).
- Native shared libraries (`::REQUIRES ... LIBRARY` for a non-`REXX` library, `::ROUTINE` /
  `::METHOD` / `::ATTRIBUTE ... EXTERNAL` naming a non-`REXX` library) → **Phase 8**.
- Everything else the tree attributes to Phase 7 is **in**: the stream model and stream builtins,
  `.File`, the standard streams and monitors, commands and `ADDRESS`, the environment/platform
  builtins, external routine resolution, the `Sys*` subset ooTest needs, the security-manager hooks
  for commands and streams, interactive `TRACE ?`.

## Rules

- **Read-only everywhere.** `/home/moritz/dev/repos/ooRexx` (the C++ tree and `build/`), and every
  file in `/home/moritz/dev/repos/ooRexx-rust-rewrite`, including `oodocs/`, `ootest/`, `rust/`. The
  only files you create are your report and files inside your own probe directory.
- **Do not run `cargo`** — no build, no test. Other agents share the target directory. Probe the
  crate with the snapshot binary:
  `BIN=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/bins/h-5bcb28edb`
  run as `timeout 20 $BIN file.rex`.
- **Oracle**, always wrapped:
  `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`
- **Every probe from a fresh empty directory** you `mkdir` under your own probe root
  (`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/survey-<AREA>/pNN/`).
  Never the scratchpad root — it is on the oracle's external-routine search path and holds stale
  `.rex` files. Absolute paths in every redirect.
- **stdout, stderr and exit status as three separate files.** Never `2>&1`.
- Before any unusual probe read `rust/corpus/oracle-crashes.txt` and **never run its entries**
  (one blocks forever). Never `select; when 1 = 0 then; when 2 = 2 then nop; end` (segfaults the
  oracle). Never `NUMERIC DIGITS` above 1000. Never `.Package~new` on a file inside the repository.
  A symbol named `x` or `b` directly followed by a quote is a hex/binary literal — use other names.
- **File-system and command probes stay inside your probe directory.** Never write, delete, rename
  or chmod anything outside it. Commands issued through `ADDRESS` are limited to harmless ones:
  `echo`, `true`, `false`, `exit N`, `cat`/`ls` of files in your probe directory, `printf`, `env`,
  `pwd`, `sleep 0`.
- **Write your report file FIRST** (a heading skeleton), then append as you go. A killed agent with
  a partial file beats one with nothing.
- **Mark every claim** as **Measured** (a probe you ran — give the program and all three
  descriptors), **Read** (cite `file:line`: docs as `oodocs/rexxref/en-US/<file>:<line>`, C++ as
  `interpreter/<path>:<line>`, crate as `rust/crates/<path>:<line>`), or **Inferred**.
- **Enumerate from the documentation's own structure** (its section and method lists), not from
  memory and not from this brief. A documented item you did not cover is listed as not covered.
- **No subagents.**
- When done, your final message is: the report path, a summary of at most 10 lines, and anything you
  could not settle.

## Report sections (all required, in this order)

1. **Documented surface** — every documented item in your area, as a table: item, doc citation,
   one-line contract.
2. **Oracle behaviour** — measured probes for each item's normal case and its edge and error cases;
   exact bytes wherever output matters.
3. **C++ mechanism** — how the oracle implements it: the states, the order of checks, the error
   numbers raised, enough that a Rust port gets the same answers. Cite lines.
4. **The crate today** — what exists, what refuses (message and rc), where (`file:line`), and the
   seams a Phase 7 implementation would plug into.
5. **Design questions** — each with options, what each costs, your recommendation, and what it costs
   if wrong. Call out any need for `unsafe` (the workspace denies it; a site needs Moritz's
   per-site approval, so prefer a safe design), any new dependency (the offline registry at
   `~/.cargo/registry/cache/*/` holds `libc`, `nix`, `rustix`, `glob`, `regex`, `filetime`,
   `chrono`, `os_pipe` among others — check the versions present), and anything platform-specific.
6. **Proposed task slices** — ordered, each independently testable, each naming its corpus
   witnesses (Rexx programs whose oracle output it must match byte for byte) and the negative or
   adjacent case paired with each.
7. **Not done / not established.**
