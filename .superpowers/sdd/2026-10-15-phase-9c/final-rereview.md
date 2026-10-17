# Phase 9c final fixes: scoped re-review

Reviewed at `48fb59b51` (fix diff `bf5224d2b..48fb59b51`), release `rexx` and `rexxc` built in a detached worktree. Oracle runs used the standard wrapper from fresh directories.

## Verdicts

1. **Entry and exit trace lines on restored programs** (finding 1): ADDRESSED.
   `trace i` program with a routine and two methods (one raising 40.1), compiled by each side's `rexxc` as `.rxo` and `-e` `.rxe`, run with the matching `rexx`: stdout, stderr and rc identical for all four images. Source run of the same agrees too. Only the Deviation 40 interleave differs on a merged pipe.
2. **`-o` / `-od` on restored programs** (finding 2): ADDRESSED.
   Identical to the oracle for: `.rex` main, `.rxo` and `.rxe` main (digits 9, `-od` directive wins), `::REQUIRES` of a `.rxo`, `CALL 'x.rxo'`, `.Routine~newFile` of a `.rxo` and of a source (from source and from restored mains), and the count spent correctly when a restored program loads first and a source later (`options.rxo` answers main 9 / req 5, `nf3.rxo` answers 5 for the source routine and 9 for the restored one).
3. **`ParseError` accessors** (finding 4): ADDRESSED.
   Mechanical: fields `pub(crate)`, `DerefMut` removed, `code()`/`sub()` added, call sites renamed, `echoing` writes through `self.0`. No behaviour change. No `code =`/`sub =` writes remain.
4. **README column** (finding 5): ADDRESSED. Names the message column and `tests/rexxc.rs`.
5. **`rexx_run` helper name** (finding 6): ADDRESSED. Renamed to `rexx` in `rexxutil_process.rs`.

`cargo test --release -p rexx-exec --test compiled`: 2 passed.

## New breakage

None.

## Out-of-scope observations

* `override_package_settings` uses `!source.is_traceable()` as the "restored" test, so it is coupled to the same predicate that gates trace lines. Fine today, worth a named predicate if a second non-traceable source kind appears.
