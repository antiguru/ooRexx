# Phase 9b1 Task 4 review (097038252..03e4cf1c7)

Binaries: head `/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9b1-t4f` (built at `b2ae5bd48`;
`git diff b2ae5bd48 cd01d7fd6 -- rust/crates` changes three doc comments only, checked), Task 3
head `rexx-run-9b1-t3f`. Crate runs `memcap 4G timeout -k 5 20`, oracle runs per
global-constraints, each from a fresh empty directory (`ls -A` empty afterwards). Probe programs
are quoted below because the scratchpad is under `/tmp`.

### Spec Compliance

- ❌ Issues found. A copy of the REXX package does not copy "like any other" (spec section 3,
  L6). Findings I1 to I3.
- ✅ Every per-package map in scout-leaks' Package group is a `PackageData` field or is left on
  `Interp` with a reason. Left on `Interp`: `required_packages` (the requires cache, spec
  section 3's last bullet), `required_paths`, `compiled_method_names`, `untranslated`,
  `library_programs`, `external_packages`, `library_codes`, `routine_objects`
  (`InstalledRoutine` -> ObjRef), and `plans`/`chunks`. All of these are compiled-program data
  keyed by ProgramId, which belongs to Task 6. Also left: `Interp::annotations` keyed by
  `Annotated::Package(Package)`, the original package's annotation table. It is per-package
  state, not on the spec's list. The implementer explains it, and a probe of copied annotations
  matched the oracle.
- ⚠️ Spec section 3 bullet 1 lists "required packages" as a field. The implementer reads that as
  the requires cache and keeps it global (Task 5 makes it weak). The oracle's per-package list is
  `loadedPackages`, which is the `imports` field. The controller should confirm that reading.

### Strengths

- Copy semantics on a program package match the oracle line for line. Probe `a_copy` covers
  `addRoutine`, `addPublicRoutine`, `addClass`, `addPackage` with a namespace,
  `setSecurityManager`, and `options` DIGITS/FORM/INITIAL/RESET/whole-string on `p2 = p~copy`.
  `p` is unchanged, and the result is identical under collect-every-allocation (281 collections).
- Every Package method reads the receiver's own body (`fields_of`, `dispatch/package.rs:164`), so
  `which_package` returning the original's program for a copy does not leak writes. I checked
  every `program_fields_mut`/`package_fields_mut` caller (identities.rs:32, :858; install.rs:255,
  :357, :604, :1139). All of them are install-time or running-program writes, and no code runs in
  a copy.
- The `PackageFields`/`PackageState` erasure is small and keeps `NativeObject` at 104 bytes. It
  adds no `unsafe`.
- Budget reproduced exactly (Q5).

### Issues

#### Critical (Must Fix)

None.

#### Important (Should Fix)

**I1. A REXX-package copy's `~addRoutine` of a library or internal Routine object is
collected while the copy holds it.** `PackageData::trace` skips `routines`
(`package_state.rs:92`, with the comment "Held by the `.ROUTINES` table"). A REXX copy never has
that table: `build_package_string_table` returns at `let program = data.program?`
(`environment.rs:1134`), so `package_string_table_for_write` answers `None` and nothing else holds
a `LibraryObject`/`InternalObject` entry. Probe, under `run_program_collect_every_alloc` with a
20-pass filler (3000 passes on the oracle):

```
c = .array~package~copy
u = .Routine~loadExternalRoutine('x', 'LIBRARY rexxutil SysSleep')
c~addRoutine('CC', u~copy)
w = .WeakReference~new(c~routines['CC'])
do i = 1 to 20; zj = .array~of('filler' i, i); end
say 'weak' (w~value \== .nil)
f = c~findRoutine('CC')
say 'found' f~class~id (f \== u) f~call(0)
```

The oracle gives rc 0 with `weak 1`, `found Routine 1 0`. The crate under stress gives rc 120
with `weak 0`, then `a message send to a value whose object is no longer live`. This surface is
new in Task 4 (Task 3 refused the REXX copy). Fix: trace the object variants in `routines` the way
`public_routines` is traced, and delete the false comment.

**I2. A REXX-package copy's `findClass`/`findPublicClass` ignores what `~addPackage` imported.**
For a package with no program, `installed_class_of` and `package_public_class_of`
(`environment/identities.rs:809`, `:823`) read only `classes`/`public_classes` and the registry,
never `merged_public_classes`. Probe:

```
q = .package~new('q', .array~of('::class qk public', '::routine qr public', 'return "qr"'))
c = .array~package~copy
c~addPackage(q)
say 'importedClasses' c~importedClasses~hasIndex('QK') c~importedRoutines~hasIndex('QR')
say 'findClass' (c~findClass('QK') \== .nil) (c~findPublicClass('QK') \== .nil)
```

The oracle gives `importedClasses 1 1` / `findClass 1 1`. The crate gives `importedClasses 1 1` /
`findClass 0 0`, rc 0 on both: a silent wrong answer. `findRoutine('QR')` and a package built
with `c` as its context both agree with the oracle.

**I3. Concern 4 measured: a merge from a REXX-package copy that redefines a REXX class keeps the
REXX class.** `merge_package` (`install.rs:818`, `:826`) takes
`package_class_table(from, true)`, which puts the registry first (`identities.rs:741`). Then
`or_insert` keeps the first entry. Probe:

```
c = .array~package~copy
c~addPublicClass('ARRAY', .list)
p = .package~new('pp', .array~of('return .array~id'))
p~addPackage(c)
say 'p findClass' p~findClass('ARRAY')~id
say 'p importedClasses' p~importedClasses~hasIndex('ARRAY') (p~importedClasses['ARRAY'] == .list)
```

The oracle gives `List` / `1 1`, and the crate gives `Array` / `1 0`. `c~publicClasses['ARRAY']`
answers `.list` on both sides, so the copy's own table and the merge disagree inside the crate.

**I4. The `imports` trace is now load-bearing and no test goes red without it.** The report says
the classes, namespaces, imports and parent traces "cannot go red here: ... every package object
is still a global root". A copy is not a global root (the progress.md ruling), so a copy that only
another package imports is held by `imports` alone. The mutant ran in a `git archive 03e4cf1c7`
copy under `/tmp` with its own `CARGO_TARGET_DIR`. It replaced `out.extend(imports.iter().copied())`
(`package_state.rs:124`) with `let _ = imports;`.
`cargo test --release --no-fail-fast -p rexx-exec --test collect_stress` gave
`40 passed; 0 failed`. This probe, under collect-every-allocation, gave rc 120 and `weak 0 1 1`:

```
p = .context~package
h = p~copy
i = p~copy
i~addRoutine('IMARK', .routine~new('im', 'return "imark"'))
wi = .WeakReference~new(i)
h~addPackage(i)
drop i
h2 = p~copy
i2 = p~copy
wn = .WeakReference~new(i2)
h2~addPackage(i2, 'NSI2')
drop i2
j = p~copy
j~addRoutine('JMARK', .routine~new('jm', 'return "jmark"'))
wj = .WeakReference~new(j)
n = .package~new('n', .array~of('return 1'), j)
drop j
do i = 1 to 20; zj = .array~of('filler' i, i); end
say 'weak' (wi~value \== .nil) (wn~value \== .nil) (wj~value \== .nil)
say 'import' h~importedPackages~items h~importedPackages[1]~routines~hasIndex('IMARK')
say 'ns' (h2~findNamespace('NSI2') \== .nil)
say 'parent' (n~findRoutine('JMARK') \== .nil)
```

Restored by an exact edit (`cmp` against the worktree file: identical). With the trace restored
the probe passes under stress (rc 0, 126 collections). It also matches the oracle with a
3000-pass filler: `weak 1 1 1`, `import 1 1`, `ns 1`, `parent 1`. The `parent` line (a copy held
only as another package's parent) is witnessable the same way, though I did not mutate it.
`namespaces` cannot be witnessed on its own: `addPackage` writes the same object into `imports`
too. The global constraint asks for a red test for every new root. Fix: add this program, or its
import and parent halves, to `collect_stress`.

#### Minor (Nice to Have)

- `options_armed` (`lib.rs:2403`) is process-wide. One `::OPTIONS` anywhere (library `.orx`
  files included, if any carry one) arms the downcast in `options_of` for every package entry,
  which is the same shape as the reqstr latch. The budget numbers show no cost today.
- `is_rexx_package` (`identities.rs:801`-`:803`) uses `is_none_or`, so a non-package answers
  "REXX package" and reads the registry. Every caller validates first through `fields_of`, but the
  silent default is a trap for a future caller. `.is_some_and(|data| data.program.is_none())`
  says what is meant.
- Report concern 4 is now measured (I3). The report's "not measured on the oracle" should be
  replaced.

### Answers to the lead's questions

1. Completeness: see Spec Compliance. No per-package mutable map is left on `Interp` except the
   original's annotation table, which has a stated reason.
2. Copy semantics: program-package copies match the oracle (probe `a_copy`, stdout identical
   apart from line 17, which is I2). REXX-package copy: I1, I2 and I3. Copies of a Package
   subclass instance were witnessed by the implementer's `subclass` line, and my probes did not
   repeat it.
3. Downcast: sound without `unsafe`. `PackageState::downcast_ref` (`rexx-core/src/body.rs:219`)
   upcasts `&dyn PackageFields` to `&dyn Any` and calls `Any::downcast_ref`, and `PackageData` is
   the one implementor. A wrong type is loud at the method surface (`fields_of`,
   `dispatch/package.rs:164`: `a value that is not a package`) and a panic in
   `package_fields_mut` (`package_state.rs:185`, reached only after `package_object` built the
   body). It is a silent `None`/skip at internal sites (`install_security_manager`,
   `add_imported_package`, `merge_package`, `is_rexx_package`), each after validation.
   Hot reads: `options_of` sits behind `options_armed` and `effective_security_manager` behind
   `security_armed`. Both latches only ever set (`security.rs:57`, `options.rs:508`,
   `install.rs:252`). The package-local read sits behind `package_locals`. The claim holds, with
   the coarseness noted under Minor.
4. GC: the `PackageData::trace` destructure is exhaustive. Every ObjRef-holding field is traced
   except `routines` (I1). The stress probe for local, `.ROUTINES` table, constants, imports with
   a namespace, and parent gave rc 0 with output identical to the oracle (155 collections).
   Roots moved from lib.rs: the implementer's seven mutants cover local, tables, main_routine,
   constants, security_manager, public_routines and merged_public_routines. I re-ran none of
   those. Imports has no red test (I4). Parent has none either.
5. Perf: `timeout 590 bench-programs/callgrind.sh -B 0.5 -j 12 -p "startup pkglocal fibcall
   extcall intcall" -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9b1/task-4-review
   pin=.../rexx-run t3=.../rexx-run-9b1-t3f head=.../rexx-run-9b1-t4f`, exit 0, three rounds,
   spread 0.0005% or less.

   | program | pin | t3 d% | head d% |
   |---|---|---|---|
   | startup | 119095643 | +0.2876 | +0.3219 |
   | pkglocal | 4185405401 | -2.6189 | -6.8709 |
   | fibcall | 9109404615 | -3.3903 | -3.4489 |
   | extcall | 8975966670 | -13.1310 | -13.2641 |
   | intcall | 839772904 | -10.7271 | -10.7938 |

   Startup's +0.3219% belongs to earlier work. Task 4 adds 40,836 Ir (119438159 to 119478995),
   which is +0.034% of the pin. The rest was there at Task 3's head (+0.2876%), and spec-review
   M7 already records +0.2930% at the 9b close.
6. Dangling: a mutated copy (routines, classes, local, options, security manager, imports) and a
   REXX copy, both dropped and then collected under stress, leave the original's answers intact
   (rc 0, 1534 collections, same stdout as the oracle). Nothing keys a copy by ObjRef or
   ProgramId: `program_packages` holds originals only, and `package_local_view`
   (`environment.rs:771`) compares a generation-carrying ObjRef. The one dangling reference found
   is inside a live copy (I1). `routine_objects` keeps a `Routine~new` object that a dead copy
   added (a leak, not a dangling entry, and Task 6's).

### Assessment

**Task quality:** Needs fixes

**Reasoning:** Program-package copies and the body refactor are correct, oracle-identical and
within budget. The newly admitted REXX-package copy has one GC hole (I1) and two lookup
divergences (I2, I3), and the imports trace that copies made load-bearing has no red test (I4).
