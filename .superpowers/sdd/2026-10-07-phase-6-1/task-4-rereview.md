# Task 4 re-review, fix round 1: 3c87b21d9..2a03b64c1

Ours: `rexx-run` (release) built from `git archive 2a03b64c1 rust interpreter`, in its own target
dir, with one `Compiling rexx-exec` line. Collect-every-alloc: the review's `zz_probe` harness, copied
into the scratch tree only. Oracle: the standard wrapper. Every probe ran from fresh empty
directories through `cmp.sh`, which runs ours plain, ours under `REXX_SWITCH_MODE=every`, and the
oracle. Probes and outputs are in
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/t4rr/`
(`probes/`, `out/<probe>/`). The scratch target dirs are deleted.

### Finding Verdicts

- **Critical 1: `.context~executable` in a `Directory~setMethod` method panics after a
  collection.** NOT ADDRESSED.
  - The review's own probes now agree with the oracle on all three descriptors, plain and under
    collect-every-alloc: `f6_dir_setmethod_alloc` (200000-iteration loop), `f2`, `f3`, `f5` and
    `s1_stress_mix`.
  - The variations the lead named also agree, plain and under collect-every-alloc:
    - `c1`: a source string, `gc 'force'`, two sends, identity, then `drop` and a third send.
    - `c2`: an array source, then an allocation loop.
    - `c3`: an unscoped `.method~new` object, and a scoped `.u~method('M')` dropped before
      `gc 'force'`.
    - `c4`: a replace, then `unsetMethod`, then `setMethod(name, .nil)` (93.974 on both), each
      followed by a forced collection and a send.
  - The defect still exists when the running entry's own `Method` object leaves the table while it
    runs. Its activation's `cold.executable` is then the only reference, and that does not root
    it. `Activity::object_roots` (`activity.rs:612-647`) roots a running or suspended activation's
    `context_object`, `replied` and `condition()` only. `Activation::object_roots`
    (`activation.rs:1016`, which does read `cold.executable`) is not called for them.
    `run_stored_method` (`dispatch/hash.rs:971`) pushes no temp, while `native_run` does
    (`object_protocol.rs:715`).
  - I instrumented a scratch copy to show `.context~executable` answers the dead handle itself. On
    `c13`, `invoke_executable` printed `entered=true body=true`, and neither the standing branch nor
    the rebuild ran.
  - Programs, each oracle rc 0:
    - `c5_dir_self_unset.rex`, rc 101 in both engine modes: the program
      `d~setMethod('x', 'self~unsetMethod("X"); call gc "force"; e = .context~executable; call gc "force"; f = .context~executable; return (e == f) e~class~id e~scope e~source[1]')`
      followed by `say d~x`. Ours panics with
      `panicked at crates/rexx-exec/src/dispatch.rs:1667:9: a live value`.
    - `c13_dir_unset_gc_exec_src.rex`: `self~unsetMethod("X"); call gc "force"; return .context~executable~source[1]`.
      Ours: rc 120, `a message send to a value whose object is no longer live`.
    - `c18_dir_unset_alloc.rex`: the same shape with no `gc` call, only an allocation loop between
      the unset and `.context~executable~source~items`. rc 120, the same message.
    - `c19_dir_replace_alloc_class.rex`: the entry replaces itself (`self~setMethod("X", "return 2")`)
      then loops. rc 120.
    - `c10_dir_replace_gc.rex`: the replace with `gc 'force'`. rc 120.
    - `c17_dir_unknown_unset_gc.rex`: an `UNKNOWN` entry that unsets itself. rc 120.
    - `c9_dir_unset_nogc.rex` agrees plain but fails under collect-every-alloc: rc 120, the same
      message.
  - The report says the rebuild "is still reachable after the Directory fix: a method that unsets
    its own entry while it runs, then a forced collection". That holds for `Object~setMethod` (`c6`
    reaches the rebuild). For a `Directory` entry, the same shape answers a swept handle instead.
  - The original panic site is fixed (see below), so the class_owns panic no longer fires. But the
    finding's observable remains: a panic, or a loud refusal, from `.context~executable` in a
    `Directory~setMethod` method once a collection has run. The new witnesses do not cover it:
    `directory_setmethod_executable_collected.rex` keeps the entry in the table.
- **Important 1: the wrong object for a scoped Method object.** ADDRESSED.
  - `dispatch/hash.rs:1933-1955` stores `scoped_method(.., .nil)`'s object, and
    `run_stored_method` answers it through `invoke_executable` (`object_protocol.rs:733-752`).
  - `i1_kinds.rex` covers a Directory, a Properties, a StringTable (97.1 on `.nil~scope`, the same
    on both) and `.local`, plus a Directory subclass. For each it checks a scoped Method object, an
    unscoped one and a source string, two sends each, identity against the original and between the
    sends, scope, and the text entry after `gc 'force'`. Identical to the oracle, and the same under
    collect-every-alloc (328 collections).
  - `i2_local_env.rex` (`.local` and `.environment` entries, read back through `.zz` and `.yy`)
    and `f1` are identical too.
- **Minor 1: false Directory clause in `Loud::object_method`'s doc.** ADDRESSED. `lib.rs:597` now
  names only `Array` and `String`.
- **Minor 2: stale exclusions record.** ADDRESSED.
  - `phase-4-exclusions.txt:5574` is checked by running the record's own two-line program
    (`x1_excl_record.rex`, rxregexp resolvable). Both rc 168, the same stdout. Our stderr differs
    only by the one line `*-* Compiled method "UNKNOWN" with scope "Directory".`, as the record says.
  - "The same line appears for a body compiled from source text": `x2_excl_source.rex` gives
    rc 215 on both, and the only difference is that line.
  - The witness `setmethod_loaded_method.rex` is identical to the oracle, rc 168. The `.env`
    sidecar mechanism exists (`tests/support/sidecar.rs`).
- **Minor 3: unrecorded oracle crash.** ADDRESSED.
  - Entry 30 (`oracle-crashes.txt:1188`) follows the file's rule: it was not re-run, and the entry
    states the wrapper, the 5-of-5 count, that the measurement came from the trapping probe `e7_5`,
    the date, the crate's answer and the ticket status.
  - I ran ours only on the entry's bare program: 93.961, rc 163, as stated.
  - For one wording problem in the entry, see New Breakage.
- **Minor 4: MAKESTRING answering a non-string.** ADDRESSED.
  - The change is `class_protocol.rs:308-311`. `m1_makestring.rex` covers `Method~new` and
    `Routine~new`. `m2_makestring_inst.rex` covers `self~setMethod`, `run` and `Directory~setMethod`.
  - Answers covered: `.nil`, a string, an Array, a user object with a `STRING` method, a number, and
    no MAKESTRING.
    - Agrees with the oracle on stdout, stderr and rc for every answer, every taker.
    - The user `STRING` method is not sent on either side: `[an US]`.
    - The commands run on both sides: `/bin/sh: 1: AN: not found`.
  - A String-subclass answer could not be compared: ours stops at the pre-existing
    `method "NEW" of class "MYSTR" is not implemented (Phase 9)` (spec R2).
- **Minor 5: witness gaps.** ADDRESSED.
  - All nine new witnesses are identical to the oracle in both engine modes, and I read each
    stdout:
    - `directory_setmethod_executable_collected`
    - `directory_setmethod_executable_scoped`
    - `setmethod_unset_while_running_executable`
    - `method_source_makestring_object`
    - `method_source_shapes`
    - `load_package_source_shapes`
    - `program_routine_call_more`
    - `program_routine_call_signal`
    - `setmethod_loaded_method` (run separately with the oracle's lib path)
  - Each header claim is visible in its output.
  - `h1` and `h2` (concurrent) remain probes, as the report says.
- **Minor 6: `borrowed_run_row` and `load_package_object_source`.** ADDRESSED. The unsent `sm` line
  is gone. `load_package_object_source.rex:9` adds `exit`, and the output is now the 88.913 line,
  `1`, `ok2`. Both are identical to the oracle.
- **Minor 7: `routine_object_running` scan on every `::ROUTINE`.** ADDRESSED.
  - `dispatch/context.rs:435-439` scans only when the directive's `clause_span` is empty.
  - `r1_routine_inner.rex` checks identity three times, with a `gc 'force'`, for a `::ROUTINE`
    inside `.routine~new` source and a package `::ROUTINE` compared with `findRoutine`. Identical
    to the oracle.

### New Breakage in the Fix Diff

1. **Minor: the rebuilt executable has no stable identity across a collection.**
   `environment/identities.rs:267-322`.
   - The rebuild mints a fresh object that only `executable_objects` and the caller's variables
     hold. A second `.context~executable` after that object is dropped and collected answers a new
     one.
   - `c6_obj_self_unset.rex`, both rc 0: `h = e~identityHash; drop e; call gc "force"; f = .context~executable; f~identityHash = h`.
     The oracle answers `1` and ours `0`.
   - The path is the degenerate one the round made reachable on purpose (an entry unset while it
     runs). Stdout differs silently.
   - It has the same root as Critical 1: nothing ties a running activation to its executable.
2. **Minor: false clause in `oracle-crashes.txt` entry 30 (`:1198`).** It reads "the instance's
   `MAKESTRING` answers nothing". The entry's class `ba` defines no `MAKESTRING`; the
   `REQUEST('STRING')` answer is `.nil` because there is no such method.
3. **Minor: dead classes in a witness.** `method_source_shapes.rex:22-35`: the classes `t` (its
   `rr`), `ba` and `bs` are never used. `ba` is entry 30's crashing class, carried over from the
   probe. The header's "a sparse array" is `.array~new(3)`, which has no items at all: the output is
   `ok 0 [The NIL object]`.

Other checks on the diff:

- `run_method` is now private.
- `invoke_executable` is shared by `run` and the Directory path.
- `executables_rebuilt` is cfg(test), and the `Interp` destructure lists it.
- The `refusal-sites.tsv` line drift (`:333` to `:336`) matches the code.
- No other writer stores a non-Method object into a Directory's METHODS half that
  `run_stored_method` would now treat differently. The only other writer is
  `directory_put_method_value`, which stores `.environment`'s `LOCAL` (`environment.rs:442`), and
  `LOCAL` has no executable record.
- `o1_run_gc.rex` (`run` of a source that forces collections and then reads
  `.context~executable`) agrees. `native_run`'s temp keeps that path rooted.

### Out-of-Scope Observations

- The original review's Strength "`ObjectMethod.executable` and `ActivationCold.executable` are
  traced" is half false. `cold.executable` is traced only where `Activation::object_roots` runs,
  which is not the running-activation walk (`activity.rs:612-647`). Whether any other `cold` field
  read through a running activation (for example `auto_expose`'s `owner` and `scope`) depends on
  being rooted elsewhere: not checked.

### Verdict

**Fix round:** Findings remain open: Critical 1. A `Directory~setMethod` entry that unsets or
replaces itself while it runs still panics (rc 101, `c5`) or refuses loudly (rc 120: `c10`, `c13`,
`c17`, `c18` with no `gc` call, `c19`; `c9` under collect-every-alloc) once a collection runs. The
cause is that `cold.executable` is not a root for a running activation. Important 1 and Minors 1-7
are addressed. New Breakage: three Minors, none Critical or Important.
