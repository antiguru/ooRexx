# Corpus probe, 2026-10-17: PPWizard and the Rexx Parser

This probe was taken for Phase 9d scoping. Moritz then moved the corpus to Phase 10. Nothing
fetched is committed. The probe ran in the session scratchpad, with the oracle at `build/` (5.3.0)
on one side and this crate's `rexx` built at `71ab69cfe` on the other.

## PPWizard 22.148

`http://dennisbareis.com/zips_fw/ppwall.zip` is 122,074 bytes, with sha256 beginning
`c538b4d899a88f61`. The `ppwizard.rex` inside has sha256 beginning `d6853c2959ab8d75`.

The input was the unpatched `ppwizard.rex tryme.it`, run from a fresh directory holding
`ppwizard.rex`, `tryme.it` and `htmlpre.ih`, 1 run a side:

| side | rc | first failure |
|---|---|---|
| oracle | 215 | 40 at `value(arg(1),,RexEnvVarPool)` with `RexEnvVarPool = 'OS2ENVIRONMENT'` (line 812), then NOVALUE and 41 cascades |
| this crate | 120 | `routine "RXFUNCADD" is not implemented (Phase 10 Rxapi)` |

The cause on the oracle is PPWizard's interpreter detection: any interpreter but Regina counts as
OS/2. The two-line patch and the patched oracle's `tryme.htm` hash, measured 2026-08-12, are in
`docs/superpowers/plans/ppwizard-gate.md`. Re-derive both before relying on them.

## The Rexx Parser

`https://github.com/JosepMariaBlasco/rexx-parser` (Apache 2.0), commit
`33cf252de3a7ca99b0dbe89a8fe2adb81c9c824b`.

`tests/RunTests.rex` aborts on the oracle before any suite runs: "revision levels differ!
Resource: 13237 Rexx: 0". Each suite was therefore run standalone from `tests/`, with
`PATH="framework:suites:../bin:$PATH"`, and with each side's `rxregexp.cls` copied into
`tests/framework/`. The oracle's copy is `build/bin/rxregexp.cls` and this crate's is
`extensions/rxregexp/rxregexp.cls`.

On the oracle, `rexx-parser-oracle.tsv` gives each suite's rc and its "Tests ran" line. Five suites
end rc 213 before printing results. The rest end rc 0, some with failures or errors of their own.

On this crate:
* Every suite ends rc 158 at `::METHOD INIT EXTERNAL "LIBRARY rxregexp RegExp_Init"` (98.903)
  unless the oracle's `build/lib` is on `LD_LIBRARY_PATH`, as the ooTest harness sets it.
* With it, `BIFs.testGroup` ends rc 120 on `routine "RXFUNCADD" is not implemented (Phase 10
  Rxapi)`.
* Nothing further was measured.
