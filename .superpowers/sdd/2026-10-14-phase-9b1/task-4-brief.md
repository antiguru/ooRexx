### Task 4: Package is a heap object holding its state

**Files:** a new body variant or a new body type for packages in `rust/crates/rexx-core`, `lib.rs`
(`Interp`'s per-package maps, `object_packages`, `package_object`), `install.rs`, `environment.rs`,
and every reader of the per-package maps listed under "Package" in scout-leaks' side-map section.

**Produces:** a Package body reachable from the package ObjRef, holding the mutable per-package state
as fields, plus the accessor every former map reader uses. Task 5 makes it collectable, and Task 6
relies on it holding the program handle.

- [ ] **Step 1:** Choose and record how the Package body refers to its compiled program across the
      `rexx-core`/`rexx-parse` boundary (spec section 3). The options: a generic or `Any` payload, a
      ProgramId field whose lifetime Task 6 owns, or moving the handle type. Measure the choice on
      the package-entry path (`lib.rs:2445`, `:2458`, `:2537`, `:2561`, `call.rs:966`,
      `dispatch.rs:3209`) with startup, pkglocal, fibcall and extcall before writing the rest.
- [ ] **Step 2:** Move the per-package state into the body as fields: options, security manager,
      routines (all four maps), classes (all four maps), namespaces, locals, imports, parents,
      required packages, the routine-object table, `package_tables` and `constant_values`. Delete
      each map from `Interp` as its readers move. `object_packages` becomes the body itself.
- [ ] **Step 3:** `~copy` is a field copy sharing the program. Witness Review Focus 3, L5 (copy of a
      Package subclass instance) and L6 (copy of the REXX package) against the oracle. Measure the
      oracle first.
- [ ] **Step 4:** Shrink `ALLOWED` ("a package object this crate did not build" and the REXX-package
      copy refusal go). Measure the full bench set against the pin, because packages are on every
      call. Per-task check, plus the suite groups `base/class/Package`, `base/directives/REQUIRES`
      and every group whose name contains `Package`, `ROUTINE` or `METHOD` (list them with the
      command in the report). Commit.

## Review Focus

1. **A package collected while still in use.** A package whose only reference is a running
   activation, a `.context~package` held in a variable, a class or routine defined in it, or another
   activity's frame. Expected: it is never collected while any of these reaches it. Task 5 pins each
   shape under `collect_stress`, and runs one from two activities under the scheduler's simulation
   mode.
2. **Re-requiring after collection.** `::requires 'x.cls'`, drop every reference, collect, then
   require it again. Expected: it loads again, and the oracle's prolog-run count and class identity
   answers hold (measure the oracle first). Task 5 pins it.
3. **A copied package mutated.** `p2 = p~copy` then `p2~addRoutine`, `p2~addClass` and
   `p2~setSecurityManager`. Expected: `p` is unchanged, `p2` changed, as on the oracle. Task 4 pins
   it, including a copy of the REXX package.
4. **A stale program id.** An activation, plan cache entry or Method object whose program was freed,
   for example a Method created in a loop and dropped, while a frame from it is still on a stack
   (`.context~stackFrames` held in a variable). Expected: never a wrong program, never a panic. Task 6
   pins it with a test that reuses freed slots under `collect_stress`.
5. **Value-based identity through collections.** A plain string stored in an IdentityTable, a Set and
   a WeakReference, then read back after collections. Expected: the held value's `identityHash` is
   stable and the WeakReference does not clear while held. Task 1 (witness) and Task 2 (L1) pin it.

---
