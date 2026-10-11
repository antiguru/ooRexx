# Scout C: the four binaries Phase 9 ships

Read `rules.md` in this directory first. Scratch directory: `/tmp/claude-1000/p9scope/c/`.
Report: `.superpowers/sdd/2026-10-10-phase-9-scoping/scout-c-report.md`.

Row 9 says `rexx`, `rexxc`, `rxqueue`, `rxsubcom` ship, and the plan's file tree puts them in a
`rexx-cli` crate that does not exist yet. The roadmap's D2 says `.rxo`/`ProgramMetaData` is "its own
Phase 9 sub-problem". D7 (RXAPI) is reopened and undecided, and Phase 10 executes it. Survey each.

1. **`rexx`.** The oracle's command line (`interpreter/platform/unix/` and its `rexx` main, plus
   `oodocs/` if it documents it): options, how arguments reach `PARSE ARG`, how a program file is
   found, stdin (`-`), `-e`, exit status rules, `.rxo` loading. Compare with
   `rust/crates/rexx-exec/src/bin/rexx-run.rs`. List each gap. Find what in this tree already
   depends on the name `rexx-run` (tests, scripts, records) so a rename's blast radius is known.
2. **`rexxc`.** What it does in the oracle (`RoutineClass::save`/`restore`, `ProgramMetaData`,
   `Envelope`): the file format, `-s` syntax-check mode, the banner, how the oracle's `rexx` detects
   and loads a compiled file. What this crate has for any of it (grep `rexxc` in `rust/` and
   `phase-4-exclusions.txt`). Which ooTest groups or corpus files use `rexxc` or compiled programs.
   State the two compatibility questions D2 raises (crate reading oracle `.rxo`, oracle reading
   crate `.rxo`) and what evidence bears on them. Also: `corpus/errors/parse-errors.tsv` is driven
   through the oracle's `rexxc`; say how.
3. **`rxqueue` and `rxsubcom`.** What each does in the oracle and what it needs from RXAPI (the
   daemon, its protocol, `docs/superpowers/plans/rxapi-protocol.md`). Can either ship meaningfully
   before D7 is decided? Which ooTest groups exercise them?
4. **Embedding API.** Row 10's text says `RexxCreateInterpreter` and `RexxStart` are "Phase 9's
   embedding API". Find what of that the `rexx-api` crate exports today and what `rexx`'s oracle
   main uses of it, so the spec can say whether `rexx` is built on the embedding API.
5. Rexx-level surface tied to the binaries: `PARSE SOURCE`, `.rexxinfo`, the `ADDRESS` environment
   names a `rexx` process starts with, `SYSTEM`/`COMMAND` defaults. Note differences you can probe
   cheaply against the oracle versus `rexx-run`.
