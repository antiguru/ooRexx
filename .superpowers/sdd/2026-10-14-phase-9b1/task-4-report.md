# Phase 9b1 Task 4 report: Package is a heap object holding its state

## Step 1: how the Package body refers to its program

**Choice: the body holds the `ProgramId`, inside an erased payload.** `rexx-core` gains a
`PackageFields` trait (`trace`, `clone_fields`, `Any` as a supertrait) and a `PackageState` box over
it. A plain package keeps it in `NativeObject`'s `Bound` slot (boxed, so every other native object
stays 104 bytes), and a `Package` subclass instance keeps it as `NativeState::Package`.
`rexx-exec`'s `PackageData` (`src/package_state.rs`) is the one implementation: its `program:
Option<ProgramId>` names the compiled program, `None` for the REXX package, and every copy holds the
same id. `Interp::programs` stays the strong holder, and Task 6 owns its lifetime (prune at sweep by
the ids live package, method and routine objects hold).

The state has to be erased whatever the program handle is: its fields are `rexx-exec` types
(`MergedRoutine`, `PackageOptions`, `ProgramId`). So the three options differ only in what the
erased payload carries:

| option | what the body holds | why not |
|---|---|---|
| ProgramId (chosen) | `Option<ProgramId>` | |
| `Rc<Program>` in the payload | the handle itself | The call path reads `programs[id]` from the activation's id either way. A held `Rc` makes the table a second holder that Task 6 must make weak, and a weak upgrade per call costs on the path that has the headroom to lose. |
| move the handle type | `rexx-core` depends on `rexx-parse`, or `Program` moves | A crate-graph change for a field `rexx-core` never reads. |

The hot path is unchanged in shape: the activation keeps `program_id` and `Rc<Program>`, so
`call.rs`, `dispatch.rs` and `lib.rs`'s `Rc::clone(&self.programs[..])` read what they read before.
The per-entry read that moves is `options_of` (`start_from_package`, the requires prolog check):
`Interp::program_packages[id]`, then the heap slot, then a downcast, where it was a
`FxHashMap<ProgramId, PackageOptions>` probe. `program_packages` replaces `package_objects`
(`Package -> ObjRef`); `object_packages` is gone, the body's own `program` field answers it.

**Measured** with the options, initial options and package identity moved (the rest still in maps),
from `rust/`:

```
timeout 600 bench-programs/callgrind.sh -B 0.5 -j 8 -p "startup pkglocal fibcall extcall" \
    -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9b1/task-4-step1 \
    pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run head=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9b1-t4a
```

exit 0. Records in `docs/superpowers/records/2026-10-14-phase-9b1/task-4/step1-callgrind/`.

| program | pin | head | whole d% | -libc d% |
|---|---|---|---|---|
| startup | 119095470 | 119445463 | +0.2939 | +0.4184 |
| pkglocal | 4185405401 | 4091785511 | -2.2368 | -3.7204 |
| fibcall | 9109404880 | 8805403334 | -3.3372 | -2.2911 |
| extcall | 8975966670 | 7785339747 | -13.2646 | -1.7880 |

Against Task 3's head (`task-3-fix1`), startup moved +7,304 Ir (119438159 to 119445463). All four
are inside +0.5% of the pin.
