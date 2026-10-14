# Phase 9b1 scout: representation-leak inventory

Read-only scout, 2026-10-14, at `ce3793971` (branch `plan/rust-rewrite`). No source edited.

**Binaries.** Oracle `/home/moritz/dev/repos/ooRexx/build/bin/rexx` (5.3, RelWithDebInfo). Crate
`rust/target/release/rexx-run`, built 2026-10-13 22:19. The last commit touching `rust/crates` is at
22:18:19 that day (`git log -3 --format="%h %cd" -- rust/crates`), so the binary is HEAD's crate
code. Every probe ran under `memcap 4G` and `timeout 20`, from its own fresh empty directory. The
runner wrote stdout, stderr and rc separately for each side. Probe sources are inlined below. The
working copies are in the session scratchpad (`.../scratchpad/leakscout/p*/`), which is not durable.
`corpus/oracle-crashes.txt` was checked first. No probe uses a listed shape: no `ENDLOCAL` past a
restore (entry 10), no `~copy` of a Stream (13), no foreign native method (33), and no raise inside
`SysStemSort` (37).

**A trap I hit and corrected.** `x~identityHash = y~identityHash` under DIGITS 9 compares two
addresses numerically and answers 1 for different objects. Deviation 4 warns about exactly this.
Every identity probe below uses `==`. The first batch used `=` and was discarded.

---

## How the candidates were derived

(a) Refusal sites. `refusal-sites.tsv` names constructors, not message text. The free-text
constructors (`receiver_class`, `unknown_receiver`, `object_method`, `setup_method`) take their noun
at each call site. So the derivation is the TSV, then the call sites of those constructors:

```
grep -P '^Loud\t' rust/corpus/refusal-sites.tsv | cut -f2-8
grep -vP '^#' rust/corpus/refusal-sites.tsv | grep -inE "copy|package|identit|weak|objectname|routine object|row|not build|no longer|Deviation"
grep -rhoP --include=*.rs 'Loud::(receiver_class|unknown_receiver|object_method|setup_method|native_method|operator_operand)\(\s*"[^"]*"' rust/crates/rexx-exec/src | sort | uniq -c | sort -rn
grep -rn --include=*.rs -A2 -E 'Loud::(receiver_class|unknown_receiver|object_method)\($' rust/crates/rexx-exec/src | grep '"'
grep -rn --include=*.rs -E "did not build|does not keep|no longer live|does not build|this crate does not|crate did not" rust/crates/ | grep -v '/tests'
```

(b) Side maps:

```
sed -n '/^struct Interp {/,/^}/p' rust/crates/rexx-exec/src/lib.rs | grep -E '^\s+[a-z_]+: .*(Map|Set|Vec)<'
grep -rnE "^\s+(pub(\([a-z]+\))? )?[a-z_]+: .*(Map|Set)<[^>]*(ObjRef|ProgramId|MethodId|usize|Package|u32|u64|Box<\[u8\]>)" --include=*.rs rust/crates/ | grep -v /tests | grep -v rexx-exec/src/lib.rs
for f in message_outcomes started_messages unnotified_messages executable_sources method_flag_writes rexx_routine_objects executable_objects; do echo "$f: $(grep -rn "$f\.\(remove\|retain\|clear\)" rust/crates/rexx-exec/src | wc -l)"; done
```

Pruning was checked against `Interp::collect_now` (`lib.rs:3158`-`:3220`) and the `object_roots`
destructure (`lib.rs:2934`).

(c) Fast paths: `eval.rs` (`arith_small_int`, `arith_left_operand`, `apply_binary`,
`operator_message_receiver`), `value.rs::to_number`, `run/loops.rs::header_number`,
`dispatch/reqstr.rs::required_string_or`, `rexx-core/src/heap.rs` weak clearing, and
`rexx-core/src/handle.rs` (TAG_INT, TAG_TEXT, INLINE_TEXT = 7).

---

## Leaks, one per section

### L1. A WeakReference to a held inline string or small integer is cleared

```rexx
x = 'ab' || 'c'; n = 12345 + 1; h = copies('x', 30)
w = .WeakReference~new(x); u = .WeakReference~new(n); v = .WeakReference~new(h)
do i = 1 to 3000; z = copies('y', 100000); q = .array~new(100); end
say w~value u~value v~value
```

- Oracle (rc 0): `abc 12346 xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx`.
- Crate (rc 0): `The NIL object The NIL object xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx`. The heap referent
  (`h`) survives. The inline and tagged referents are cleared while `x` and `n` still hold them.
- Cause: `rexx-core/src/heap.rs:282`. `target_alive = self.resolve(target).is_some_and(...)`. A
  non-heap handle resolves to `None` and is read as "dead", so the first collection clears it.
  Constructor: `dispatch/construct.rs:594`.
- Reach: no ooTest test found (`WeakReference.testGroup` refers only to objects and relations). The
  corpus `lang/weak_reference_value.rex` uses `.Object~new`. Every string or integer handed to a
  WeakReference, then held, is affected.
- What the specialised form buys: nothing here. The weak cell is already a heap object.
- Size: tiny. Treat a non-heap target as alive at `heap.rs:282`, about 3 lines plus a corpus case.
  The other way, an unreferenced computed `'abc'` is then never cleared where the oracle clears it.
  That is collection timing (the GC-ordering licence), not identity. The uniform alternative
  (materialise the referent at `~new`) changes `w~value`'s identity. It is not needed.

### L2. User arithmetic and prefix operators on a `String` subclass instance are never sent

```rexx
s = .mystr~new('3')
say (s + 1) (s * 2) (-s) (+s) (s < 4)
::class mystr subclass string
::method '+'; if arg() = 0 then return 'pfx+'; return 'plus'
::method '-'; if arg() = 0 then return 'pfx-'; return 'minus'
::method '*'; return 'times'
::method '<'; return 'lt'
```

- Oracle: `plus times pfx- pfx+ lt`.
- Crate: `4 6 -3 3 lt`.
- Measured over all seven arithmetic operators (`+ - * / % // **`) and prefix `+ -`, which are all
  bypassed. Comparison, logical and concatenation operators and prefix `\` are sent on both sides.
- The same cause reaches DO. With `::method '+'` and `'>'` that print, `do i = s to 4`:
  - Oracle: `sent +` (the setup's prefix `+`), then 41.1 from the method's own arithmetic, rc 215.
  - Crate: `i 3`, `i 4`, rc 0. Nothing is sent.
- Cause: the number conversion is asked before the operator-receiver test. `eval.rs:843`
  (`arith_left_operand` calls `to_number` first; a String-subclass `Body::Instance` with text parses
  as a number). Also `eval.rs:646` (prefix, via the same helper) and `run/loops.rs:1062`
  (`header_number_body`: `to_number`, then `operator_message_receiver` only on failure).
  `apply_binary` (`eval.rs:1311`) asks `operator_message_receiver` first, which is why comparisons
  agree.
- Reach: no ooTest file declares an arithmetic operator method
  (`grep -rlE "::method +[\"'](\+|-|\*|/|%|//|\*\*)[\"']" ootest/ooRexx` is empty). The corpus
  `lang/do_object_*` and `do_header_objects.rex` use plain `::class` receivers, not String
  subclasses, so they pass.
- What it buys: nothing for the tagged and inline paths. `arith_small_int` runs first and is
  untouched. The reorder costs one heap-body match on the general path only. Not measured.
- Size: small. Ask the operator-receiver test before `to_number` for heap `Instance` bodies, at the
  three sites. About 20 lines plus corpus cases. The DO arm of Deviation 25 then applies to these
  receivers too.

### L3. A class-side `STRING` method is not sent when a class object's string value is needed

```rexx
say .k; say length(.k); say 'x' || .k; say .k~string
::class k
::method string class; return 'KS'
```

- Oracle: `KS`, `2`, `xKS`, `KS`.
- Crate: `The K class`, `11`, `xThe K class`, `KS`. Only the explicit send agrees.
- Cause: `dispatch/reqstr.rs:198`. `required_string_dispatch` sends `STRING` only for
  `Primitive::Instance`; a class object takes `string_value_text`. This was first seen in
  `.superpowers/sdd/2026-10-13-phase-9b/task-3-review.md` ("Out of scope"). I confirmed Array,
  Directory, MutableBuffer, String and Stem subclasses with an instance `STRING`, and all agree
  (`subclass_string_kinds`).
- Reach: no ooTest or corpus file declares `::method string class`.
- What it buys: `say .Array` and every class rendering skip a method lookup. This is off the hot
  path. Not measured.
- Size: small. Send when the class's class-side behaviour resolves `STRING` to something other than
  the native. About 15 lines.

### L4. Strings and small integers have no identity of their own (Deviation 4, Deviation 31)

All probes use `==` on `identityHash`, or `.IdentityTable`:

| probe | oracle | crate |
|---|---|---|
| `x='abc'; y='ab'\|\|'c'`: same object? | 0 | 1 |
| `'abc'~copy` same object? | 0 | 1 |
| `n=7; n~copy` same? / `(2+5)~copy` same? | 0 / 0 | 1 / 1 |
| `150+0` vs `150+0` same? / `1000000+1` twice | 0 / 0 | 1 / 1 |
| `2+3` vs `1+4` / `5` literal vs `2+3` | 1 / 1 | 1 / 1 |
| `.String~new('abc')` twice same? | 0 | 1 |
| IdentityTable: `t['abc']=..; t~hasIndex('ab'\|\|'c')` | 0 | 1 |
| `parse value 'abc def' with p .`; `p` vs literal `'abc'` | 0 | 1 |
| 10-byte strings, 12-digit integers | 0 | 0 (agree) |
| `s='abc'; s~objectName='named'` | `named` | rc 120 `OBJECTNAME= on a string or a number is not implemented` |
| `longs = 'a longer string than seven'; longs~objectName='long'` | `long` | rc 120, same text |

- The divergence runs both ways. That is Deviation 4's table, re-measured on the merge side.
  Deviation 4 also records the separate side: identity-preserving no-ops, and re-evaluated
  literals of 8 or more bytes.
- Not reachable on a plain string or integer: `setMethod` and `run` (private, 97.2 on both),
  `.String~define` (98.985 on both), and `EXPOSE` (needs a method whose self is the value, and only
  `define` would give one). So object variables, own methods and guards on a value are unobservable
  today. Only identity and `objectName` are observable.
- Code: `rexx-core/src/handle.rs:16`-`:45` (TAG_INT, TAG_TEXT, INLINE_TEXT 7, SMALL_INT range
  ±2^61), `lib.rs:566` (`object_name_of_a_value`). Heap objects whose body has no name field use
  `Heap::names`, a side table (`rexx-core/src/heap.rs:85`). It is pruned at sweep.
- ooTest reach: `base/class/IdentityTable.testGroup` (fixture `"22", "2"||"2"`). Deviation 4 counts
  26 of 42 methods as the upper bound. No ooTest file calls `objectName=` on a string; the
  objectName tests use arrays and objects. The corpus routes around the merge: `lang/string_copy.rex`
  and `lang/hash_table_store.rex` deliberately use keys too long to be inline.
- What it buys: Deviation 4 records entry 59 (inline text) at −6.448% `strings` and −6.154%
  `rexxcps`. Tagged integers predate that and carry all arithmetic. Not re-measured.
- Size: large. This is 9b1's design decision (step 3): materialise on demand plus per-package
  literal pooling, against the oracle's construction-path identity. An OBJECTNAME-only fix
  (materialise into a heap string on `objectName=`) cannot reach a value held inline in another
  variable slot. That is the "hardest case" in the queued spec.

### L5. A `Package` subclass instance's copy is refused as "a package object this crate did not build"

```rexx
p = .mypkg~new('y', 'nop'); c = p~copy
say c~class~id
say c~name
::class mypkg subclass package
```

- Oracle: `MYPKG`, `y`.
- Crate: `MYPKG`, then rc 120 `a message send to a package object this crate did not build is not
  implemented`.
- Cause: `dispatch/object_protocol.rs:662` (skipped; the clone is at `:679`). A subclass instance is not `Primitive::Package`, so it
  takes the generic body clone. The clone is never entered in `object_packages`, which
  `Interp::which_package` (`environment/identities.rs:1385`) reads. The refusal fires at
  `dispatch/package.rs:129`, `:1163` and `:1179`, and at `class_protocol.rs:1132`.
- Reach: corpus `lang/package_subclass_new.rex` builds subclass packages and does not copy one. No
  ooTest file found.
- Size: dissolves when Package is a heap object (`2026-10-12-package-as-real-object.md`). A local
  patch is about 10 lines in the copy arm. Do not take it; it would add a third route into the map.

### L6. The REXX package cannot be copied

```rexx
c = .Array~package~copy; say c~name
```

- Oracle: `REXX`, rc 0.
- Crate: rc 120 `method "COPY" of class "Object" is not implemented (Phase 9)`.
- Code: `dispatch/object_protocol.rs:667`. `Package::Rexx` has no ProgramId, so `copy_package` cannot
  run. TSV row `native_method` names `.object~package~copy`.
- Size: dissolves with package-as-object.

### L7. Packages are never collected

```rexx
w = .WeakReference~new(.Package~new('src', 'nop'))
do i = 1 to 3000; z = copies('y', 100000); q = .array~new(100); end
say (w~value == .nil)
```

- Oracle: `1`. Crate: `0`. The same holds with `.context~package~copy` (`pkg_copy_weak`): oracle
  `1`, crate `0`.
- Memory, `/usr/bin/time -f %M` on the binary itself under memcap:

  | loop | oracle | crate |
  |---|---|---|
  | 20000 × `.Package~new('p','return 1')` | 21000 kB | 629024 kB |

  That is about 30 kB per package, kept for good.
- Code: `package_objects`/`object_packages` (`lib.rs:1428`, `:1431`). Each package object is a
  global root (`RootSet::add_global`, `package_root_key`, per the `object_roots` comments at
  `lib.rs:3003`). Every per-ProgramId map in section "Side maps" also holds it.
- Reach: ooTest `base/class/Package.testGroup` `testReclaim01` (`:155`) is renamed out on timeout
  (the queued spec). Sibling: `pcopyloop.rex`, about 5.8 kB per `~copy`, is in
  `2026-10-12-gc-trigger-blind-to-class-weight.md`.
- Size: this is the package-as-object item itself. Large.

### L8. Every `Method~new`, `Routine~new` and source-string `setMethod` mints a program that is never freed

| loop (20000 / 80000) | oracle | crate |
|---|---|---|
| `m = .Method~new('m', 'return 1')` | 20004 / 20084 kB | 431044 / 1658768 kB |
| `r = .Routine~new('r', 'return 1')` | 20636 / 20720 kB | 430976 / 1653804 kB |
| `self~setMethod('FOO', 'return 1')` in a method, 20000× | 20344 kB | 434132 kB |
| `interpret 'x =' i`, 20000× | 20628 kB | 21952 kB (agrees) |

- That is about 20.5 kB per executable ((1658768 − 431044) / 60000). Oracle memory stays flat.
- Correctness: `.WeakReference~new(.Method~new(...))` and `.Routine~new` both clear on both sides
  (`method_weak`), so the object goes. The ProgramId and everything keyed by it stay. Unbounded
  growth is observable only as memory, and it ends in OOM at about 200k definitions.
- Cause: `install.rs:1729`-`:1731` and `:1864`-`:1866` push onto `Interp::programs`
  (`Vec<Rc<Program>>`, append-only, indexed by ProgramId) and insert `compiled_method_names`.
  `override_package_settings` adds an `option_writes` row. The `plans` and `chunks` caches key by
  `BodyKey`, which includes the program. `package_from_source` (`install.rs:994`) does the same for
  `Package~new`.
- What it buys: a ProgramId is a `usize` index rather than an `Rc`. Not measured.
- Size: medium to large. Once the package is an object holding `Rc<Program>`, a ProgramId stops
  being a permanent identity, and `programs`/`plans`/`chunks` need either weak entries or a
  free-list. Same work item as L7. The queued package spec does not name it, so add it.

### L9. ObjRef-keyed side maps of Method, Routine and Message objects are never pruned

- `msgloop` (`m = .Message~new(o,'ITEMS'); m~send`): 20000 → 33612 kB, 80000 → 45216 kB. Oracle:
  18168 and 20432 kB. That is about 0.19 kB per message, kept for good.
- Maps with no `remove`/`retain` call (grep above): `executable_sources` (`lib.rs:1515`),
  `method_flag_writes` (`:1522`), `started_messages` (`:1529`), `executable_objects` (`:1512`),
  `rexx_routine_objects` (`:1628`). `message_outcomes` (`:1526`) has one `remove`
  (`object_protocol.rs:1164`) on one path. `collect_now` prunes `kept_strings`, guard pools,
  semaphores, scheduler senders, `class_variables`, `class_packages` and `method_objects`, and
  none of these.
- Correctness: none. Generation-bearing handles make a swept key a miss, which the `object_roots`
  comments rely on. This is memory, and it is per-object state outside the object, which is the
  rule's second clause.
- Size: small as a stopgap (prune in `collect_now` the way `kept_strings` is pruned, about 15
  lines). Uniform: move each into the object's body (a `Message` body carrying outcome and flags,
  `Method`/`Routine` bodies carrying their `ExecutableRecord`). Medium.

### L10. A copy of an internal-row `Routine` cannot be added to a package

```rexx
p = .context~package
c = .Routine~loadExternalRoutine('x', 'LIBRARY rexxutil SysSleep')~copy
signal on syntax
p~addRoutine('C1', c); say 'copy added' (p~findRoutine('c1') == c) c1(0)
exit
syntax: say 'copy' condition('o')~code
```

- Oracle: `copy added 1 0`. Crate: `copy 88.914`.
- This is the queued spec's 9b Task 9 item. `rexx_routine_objects` is a side map from Routine object
  to its internal row (`lib.rs:1628`). `object_protocol.rs:658` re-records the copy there, but
  `addRoutine`'s check reads a directive/row record (`routines` / `MergedRoutine`) that a copy does
  not carry. The exclusions row is "addRoutine REFUSES A LIBRARY Routine OBJECT".
- A user `::routine` copy added the same way agrees (`routine_copy_add`: `r` on both).
- Size: dissolves when a Routine object carries its code (row or directive) in its body. Small to
  medium on its own.

### L11. A native routine's level is not a live stack frame

```rexx
s.0 = 2; s.1 = .k~new; s.2 = 'a'
call SysStemSort 's.'
::class k
::method string
  if .local~seen \= .true then do; .local~seen = .true
    do f over .context~stackFrames; e = f~executable
      say f~type f~name~right(12) e~class~id (e~source~items); end; end
  return 'b'
```

- Oracle:
  - `METHOD       STRING Method 8`
  - `ROUTINE  SYSSTEMSORT Routine 0`
  - `PROGRAM ...exec.rex Routine 3`
- Crate: the same without the `ROUTINE SYSSTEMSORT` line.
- Cause: the crate builds a native level's frame only when a failure is blamed. See
  `dispatch.rs:3883` `blame_native_level` and `context::build_native_level_frame`. The source-less
  `Routine` for a REXX-package native level comes from `rexx_routine_executables`, a by-name map
  (`lib.rs:1453`, `identities.rs:1336`), and only on that path. A native call has no activation
  object, so `.context~stackFrames` cannot list it.
- Reach: ooTest `StackFrame.testGroup` uses `.context~stackFrames` from Rexx levels only. None
  found through a native callback.
- Size: medium. Push a native activation record that the frame list reads. The crate is single-
  activation-stack today, so this is not trivial.

### L12. SysStemSort writes back by tail; the oracle writes into the elements it captured

Not re-run here. The probe and measurement (both sides, rc 0) are in
`.superpowers/sdd/2026-10-13-phase-9b/task-3-review.md` Minor 1. A STRING method that empties the
stem mid-sort leaves `[S.1] [S.2]` on the oracle and recreates the tails on the crate. Fixing it
needs stem element (compound variable) identity, which the crate does not have: a stem is a store of
tail to value (`Body::Stem`), not of tail to variable object. No other observable of compound
element identity was found. `stem_alias` (`b. = a.`), `stem_copy` and an object tail all agree.
Size: medium, and only if 9b1 gives compound elements identity. Otherwise record it as a Deviation.

### (e) Activity- versus activation-scoped state: closed

The memory note's three cases were moved by Phase 6.1 (`66b0f6854`, "Activation layout for the
scope move"). They now sit on `Activation`: `random_seed`, `locals` (SETLOCAL) and
`elapsed_anchor`/`cached_clock` (`activation.rs:375`, `:583`, `:586`). `Activity::random_source`
(`activity.rs:243`) is only the generator that seeds a top-level activation, as the oracle's
activity does. Probes, all agreeing:

- `random_scope`: a method seeds 99, an internal routine seeds 9, main reads. Both print `518`,
  `429`.
- `setlocal_scope`: a method `setlocal()`, main `endlocal()`. Both print `1`, `0`. This is the
  memory note's own case, which used to be oracle 0 against crate 1.
- `clock_scope`: a `time('R')` in an internal routine and a method. Both print `1`, `1`.

The REPLY-seam variants were not re-probed. The queued files
`2026-10-02-elapsed-clock-per-routine-and-reset.md` and `2026-10-07-setlocal-scope.md` look stale
against this and should be checked before 9b1 claims them.

### Refusals from (a) judged not to be representation leaks

- `unknown_receiver` ("a value whose object is no longer live", "a value with no class of its own",
  "a program this crate did not load", "a routine ... with no directive/table/package entry", "an
  object this crate did not build"): the TSV row reads `agrees`, no route. These are guards on
  side-map lookups. They become unreachable rather than wrong once the maps are object fields. No
  probe reached one.
- "a stack frame / an executable / a routine object / a method object this crate did not build":
  `receiver_class` and `unknown_receiver` guards on `executable_sources` and the frame side data.
  The `.Method` copy, `define`, `run`, StackFrame copy, and executable-after-`delete`/`redefine`
  probes all agree.
- `environment_entry`, `environment_symbol` and `unreadable_collection` (`.local['STDQUE']`,
  `defineMethods(.local)`): a missing object (STDQUE, Phase 10), not a representation of one.
- `metaclass_new_reused_class` (Deviation 32), `receiver_class` "a value that is not an array/list/…"
  (Deviation 28), `string_answer_not_a_string` (Deviation 30): Deviations, cross-referenced below.

---

## (d) Deviations that stem from representation (cross-reference only)

From `docs/superpowers/plans/phase-4-exclusions.txt`, "DEVIATIONS -- permanent, chosen" (`:715`):

- **4**, object identity not modelled. This is L4, and the root of 31.
- **31**, OBJECTNAME= on a string or small integer refused. This is L4.
- **6**, a driven collection reaches a just-allocated object. Its 5j widening names `WeakReference`.
  L1 is a different mechanism: a held value cleared, not an early clear.
- **25**, DO TO/BY judges a user operator by truth value. It adjoins L2: once L2 is fixed, String
  subclasses fall under it.
- **28**, a primitive's native method on another class refused. Representation-adjacent: bodies are
  typed per primitive.
- **30**, a STRING answer with no string value refused. Not representation.
- **32**, a metaclass NEW answering a non-built class. Representation-adjacent: the crate cannot
  rewire a class in place.
- **33**, hash order (ruled). **9**, MutableBuffer storage. **1**, DO OVER stem order. Not
  representation-driven, or already ruled.

---

## Ranked by observable impact

| # | leak | what a program sees | tests that reach it | fix size |
|---|---|---|---|---|
| L1 | WeakReference to a held inline/tagged value cleared | wrong value (`.nil` for a live referent) | none found | tiny |
| L2 | user arithmetic/prefix ops on a String subclass, and DO setup `+`, not sent | wrong value, missing side effects, missing error | none found | small |
| L4 | value identity (IdentityTable, identityHash, copy) and OBJECTNAME= | wrong answer both ways; rc 120 on OBJECTNAME= | IdentityTable.testGroup (≤26 methods) | large (9b1 design) |
| L8 | Method/Routine/setMethod/Package~new never freed | unbounded RSS, about 20–30 kB each, eventual OOM | every suite run; testReclaim01 | medium–large |
| L7 | packages never collected | WeakReference never clears; memory | Package.testGroup testReclaim01 | large (package object) |
| L5 | Package subclass copy refused | rc 120 | none found | dissolves with L7 |
| L6 | REXX package copy refused | rc 120 | none found | dissolves with L7 |
| L10 | internal-row Routine copy refused by addRoutine | 88.914 where the oracle adds it | none found | small–medium |
| L3 | class-side STRING not sent | wrong rendering | none found | small |
| L11 | native routine level missing from live stackFrames | a missing frame | none found | medium |
| L12 | SysStemSort writes back by tail | different stem contents in an exotic case | none | medium, or a Deviation |
| L9 | Message/Method/Routine side-map rows never pruned | memory only (about 0.19 kB per message) | none | small stopgap / medium uniform |

## Side maps, grouped by the object they belong to

All are on `Interp` (`rust/crates/rexx-exec/src/lib.rs`) unless noted.

- **Package** (keyed by ProgramId, `Package` or name):
  - `programs` (append-only), `package_options`, `option_writes.initial` (`options.rs:479`) and
    `option_writes.rexx`, `security_managers`.
  - `routines`, `routines_written`, `package_public_routines`, `merged_public_routines`,
    `merged_public_classes`, `package_classes`, `package_public_classes`.
  - `package_namespaces`, `package_locals`, `package_objects`/`object_packages`,
    `program_routine_objects`, `package_tables`, `package_imports`, `package_parents` and
    `parented`.
  - `constant_values` ((ProgramId, directive)), `compiled_method_names`, `required_paths`,
    `required_packages` (name → ProgramId), `untranslated`, `library_programs`.
  - `external_packages` (MethodId → ProgramId), `library_codes` (row → ProgramId).
  - The `plans` and `chunks` caches (`BodyKey` holds the program).
- **Class** (keyed by class ObjRef):
  - `class_variables`, `class_packages`, `method_objects`, `rexx_class_cache`.
  - `rexx-classes` `ClassRegistry` `names`, `default_names`, `object_names`, `by_name`,
    `by_system_name` (`registry.rs:28`-`:40`).
  - `MethodDict.scope_orders` (`method_dict.rs:50`).
  - Pruned at sweep through `freed_classes`, except `by_name`.
- **Method / Routine object** (ObjRef, MethodId or row):
  - `executable_sources`, `executable_objects`, `method_flag_writes`, `rexx_routine_objects`.
  - `routine_objects` (InstalledRoutine), `library_routine_objects` (row),
    `internal_routine_objects` (Vec by INTERNAL_ROUTINES index; added in 9b),
    `rexx_routine_executables` (by name), `rexx_routine_rows`.
  - `method_bodies`, `generated_methods`, `native_externals`, `guarded_externals`,
    `library_externals`, `defined_library_codes`, `library_routine_codes`, `special_methods`.
  - `annotations`, keyed by the `environment::Annotated` identity enum. This is the "identity store"
    9b extended with `Annotated::InternalRoutine`. Annotation tables for classes are held by the
    class; the rest are globals.
- **Message object**:
  - `message_outcomes`, `started_messages`, `unnotified_messages`.
  - `scheduler.rs:273`/`:279` `message_ids` and `senders` (`senders` is pruned).
- **Stem**: `stem_exposers` (weak, pruned).
- **Any object**:
  - `Heap::names` (objectName for bodies with no field; pruned at sweep; `rexx-core/src/heap.rs:85`).
  - `guards.rs:67`/`:71` `pools` and `watches` (pruned).
  - `kept_strings`/`kept_holders` (native API copies; pruned).
  - `rexx-api` handle tables (`handles.rs:32`, `:41`).
- **Value caches with no object to live in** (inline handles): `text_numbers` (the parse cache for
  inline text, `value::TextNumbers`) and `text_scratch`. These are internal caches with no
  observable, provided L4's materialisation keeps them valid.
