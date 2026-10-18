# Task 6.1 review: external program search for code built from source

Reviewed `a38297da2..e27a7f843` (source diff: `install.rs`, `lib.rs`, `environment/identities.rs`, two corpus witnesses, `phase-9.txt`, `suite-9d-set.txt`).
Binary built at `e27a7f843` in a scratch target dir; oracle and 9c-close pin (`rexx-9c-25197002a`) run side by side from fresh directories, stdout, stderr and rc compared separately.

## Spec compliance

Compliant.
The three paths (`::REQUIRES`, external `CALL`/function call, `Package~findProgram`) now go through `Interp::resolve_in_package` (`install.rs:935`), and so does `~loadPackage`.
The 25 lines removed from `suite-9d-set.txt` are exactly the 25 `changed` rows of `t6.1/suite/changed-vs-task-4.tsv` (set `diff`, empty), and `suite-r1/changed-vs-t6.1.tsv` has no rows.
Both witnesses match the oracle byte for byte and are red on the pin (rc 213, 43.901 `rqd_own` and 43.1 `EXT_OWN`).

## Oracle rule against `resolve_in_package`

* `PackageClass::resolveProgramName` (`PackageClass.cpp:926`) and `findProgramRexx` (`:946`) run `InterpreterInstance::resolveProgramName` with the package's `programDirectory` and `programExtension`, then recurse into `parentPackage`.
  Each level is a full search: `SysSearchPath` is own dir, `.`, extension path, `REXX_PATH`, `PATH` (`SysInterpreterInstance.cpp:123`).
  The crate's `package_chain(...).find_map(resolve_search(...))` is the same shape: a full `require::search` per package, own first.
* Extension handling, `.cls` first for requires, then the package's extension, then `.REX`/`.rex`, then bare: unchanged `require::candidates`, matches `InterpreterInstance.cpp:1167`.
* Package without a package object falls back to a single search from `program_display_name`. The REXX package (`program == None`) searches globally, which every earlier level already covers, so it changes nothing observable.
* `programDirectory`/`programExtension` are extracted from the name as given (`PackageClass.cpp:433`). `program_display_name` is that name for code built from source and the resolved path otherwise. Matches.

## Probes (all against oracle and pin, `REXX_PATH` set where named)

Probe files: `<scratchpad>/rev61/w1/main/prog.rex`, `more.rex`, `sym.rex`, `w1/reg/plain.rex`.

| case | oracle | crate | pin |
|---|---|---|---|
| bare `.Routine~new`, no context, file in main dir only | main | same | same |
| bare, file in main dir and cwd | cwd | same | main |
| named `own/x.zzz`, file in own and cwd | own | same | cwd |
| named `own/x.zzz`, file in cwd and main | cwd | same | main |
| bare, file on `REXX_PATH` only | rp | same | same |
| bare with parent `par/plib.cls` | par | same | 43.901 |
| nested: `own/y.zzz` inside a bare routine whose parent is `plib` (own+par file, par-only file) | own par | same | 43.901 |
| `CALL` from `own/x.zzz`, only `c9.zzz` in own | own-zzz | same | main-rex |
| `::requires 'rq.zzz'` from `own/x` (no extension) | own | same | 43.901 |
| relative name `../own/x.zzz` | own | same | 43.901 |
| name with `'e9'x` in its directory, requires and `CALL` | ownE9 | same | main |
| `INTERPRET 'call ci'` inside `own/x.zzz`, and in main | own, main | same | main, main |
| `findProgram` from `own/x.zzz`, from bare | own path, main path | same | cwd path, `.nil` |
| function call `c3()` from `own/x.zzz` | own | same | cwd |
| `~loadPackage` from `own/x.zzz`, and through parent `plib` | own, par | same | 43.901 |
| file required from dynamic code requires a main-dir-only file | 43.901 `r11` | same | 43.901 `r10` |
| `.Package~new('rel/pk', src)` without context, `CALL c1` | 43.1 | same | main |
| **named `own/.zzz`, `CALL c9`** | **own-zzz** | **main-rex** | main-rex |

Regression: a plain program in `reg/` requiring from its own dir, from cwd, from `REXX_PATH`, with own-dir-beats-cwd, plus `CALL`, `findProgram` and `INTERPRET`, run by absolute and relative path: oracle, crate and pin identical.
A program run through a symlinked directory (absolute and relative), bare requires and `CALL`: identical on all three.

## Strengths

* One walk (`resolve_in_package`) serves all four entry points, and it reuses `package_chain` and `require::search` rather than restating either.
* The old `compiled_method_names` special case in `resolve_program_name` is gone instead of patched, and its measured fact (bare name searches cwd first) survives in the doc and holds under probe.
* The report's suite claims and red/green claims reproduce exactly.

## Issues

### Critical

None.

### Important

None.

### Minor

1. `require.rs` `program_extension` exempts the first byte of the last *component*, where `SysFileSystem::extractExtension` exempts only the first byte of the whole *string* (`SysFileSystem.cpp:256`, same for `hasExtension` at `:322`).
   A dynamic package named `own/.zzz` gets extension `.zzz` in the oracle and none in the crate, so `CALL c9` finds `own/c9.zzz` there and `main/c9.rex` here (probe row in bold, `prog.rex` T18).
   The helper predates the task, but this task is what feeds given names into it.
   Fix: test the dot's offset in the whole string (`> 0`), not in the component, and the same for `has_extension`. The doc comment there says "component" and is wrong in the same way.
2. `program_directory` uses `rfind('/')` over the whole string, but `extractDirectory` never examines index 0, so a given name `/x` has directory `/` in the crate and none in the oracle.
   Not observable without a file in `/`, so not witnessed. Worth the same one-line alignment as item 1.
3. Neither witness puts the target in both the current directory and the parent package's directory, so a mutant that walks every package's own directory before the current directory would pass both. Rows 2 and 4 of the table above show the order holds today. A fourth requires case with the file in cwd and the parent's dir would pin it.

## Assessment

**Spec:** compliant.

**Task quality:** Approved. The rule matches the oracle on every path and case probed except the dot-leading component name, which is a pre-existing helper defect newly reachable here.
