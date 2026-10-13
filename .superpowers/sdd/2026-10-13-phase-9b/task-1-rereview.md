# Task 1 re-review, fix round 1 (0368229d1..307771ede)

Verdict: approved. I1 is resolved as ruled; no new defects.

* I1: `class_in_program` (`environment.rs:612-620`) answers `Ok(None)` for `Owed(Some(_))` and `Absent`
  from the `.local`/`.environment` lookup, and still refuses `Owed(None)`. The package-local `Owed` arm
  (`:605`) is unchanged and loud. `package_find_class` is untouched in the diff. The doc comment states
  the exception.
* Unit test: `OWEDCLASS` in the package local expects `Loud`; `STDQUE` and `NOSUCHCLASS` expect `Ok(None)`.
  Doc comment updated to match.
* Witness `lang/native_find_class_stdque.rex`, run from a fresh empty dir, `memcap 4G`, oracle vs
  `target/release/rexx-run` (cargo reported it up to date): stdout, stderr and rc identical (`cmp`),
  rc 165, `Error 91.999:  Message "FINDC" did not return a result.` The witness is registered in
  `phase-9.txt`; its sourceline file is present.
* Not re-run: the mutation (reported by the implementer) and the gate commands.

Open findings: none new. M1 remains queued, M2 accepted.
Note (not a finding): the `Owed(Some)` => none answer also applies to an owed `.environment` entry
with an owner, since the lookup spans both scopes; no such entry is known to be a class.
