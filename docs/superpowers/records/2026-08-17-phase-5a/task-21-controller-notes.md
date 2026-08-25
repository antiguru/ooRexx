# Task 21 controller notes

Measured by the controller at `4a519518d`, immediately before dispatch. Where these disagree with
the brief, these win: they were run today against the shipped oracle and the shipped crate.

## 1. The brief's headline verification line does not reproduce as written

The brief says `.Array~define("ZORK", .methods~z)` is oracle rc 158 with `98.985`. **On its own it is
not.** A file containing only that line is oracle **rc 159**, error **97.1**, `Object ".METHODS" does
not understand message "Z"` -- `.METHODS` holds the *running package's own* methods, and a file that
declares none has none, so the argument fails before `DEFINE` is ever sent.

The brief's stated result needs the method to exist:

    .Array~define("ZORK", .methods~z)
    ::method z
      say "hi"

That file is oracle **rc 158**, stderr opening `       *-* Compiled method "DEFINE" with scope
"Class".` then the source echo, then `Error 98 running <path> line 1:  Execution error.` and
`Error 98.985:  User additions are not allowed to the REXX language classes.`

`~delete` behaves the same way: `.Array~delete("ID")` in a file with any `::method` is rc 158 with the
frame `       *-* Compiled method "DELETE" with scope "Class".` and the same `98.985`. Take the frame
line's method name from the message actually sent, not from a table of five copies.

## 2. `.context~package~name` answers the program's absolute file path

Measured: a file at `<dir>/p6.rex` doing `say .context~package~name` prints `<dir>/p6.rex`, rc 0.
That is not a fixed string, and the corpus compares stdout byte for byte across checkouts.

**No corpus program of this task may print `.context~package~name`.** The reachability of
`.context~package` is witnessed by something path-independent instead. Two that are measured to work
on the oracle at rc 0:

* `say .context~package~class` prints `The Package class`
* `say .context~package == .context~package` prints `1`

The identity row is the stronger of the two: it fails against an implementation that mints a fresh
Package object per send, which is exactly the silent wrong answer Task 20 shipped and had to fix for
`~method`. Build the accessor so that row passes, and treat a fresh-object-per-send implementation as
the defect it is rather than as an unobservable difference.

## 3. Crate state at dispatch, measured, all rc 120 loud

* `.Stem~method("==")` / `.VariableReference~method("==")` -- `environment symbol ".STEM" is not
  implemented (Phase 5)`; oracle prints `The NIL object` twice at rc 0.
* `.Queue~method("APPEND")` and `.Queue~method("SORT")` -- `environment symbol ".QUEUE" is not
  implemented`; oracle is `a Method` rc 0 and rc 159 `97.1` respectively, the latter with the frame
  `       *-* Compiled method "METHOD" with scope "Class".`
* the lock probe -- `method "DEFINE" of class "Class" is not implemented (Phase 5)`; `~delete` the
  same under `DELETE`.
* `.context~package` -- `a message send to one of the interpreter's own objects is not implemented`.

**`.Array~package~name` already answers `REXX` on the crate at rc 0**, agreeing with the oracle, and
`.methods~z` already resolves -- the lock probe's crate refusal falls on `DEFINE`, past its argument.
Neither needs building.

## 4. Where the pieces already are

* The native method table is `crates/rexx-exec/src/dispatch.rs`, the `("Class", "PACKAGE",
  Arity::Fixed(0), native_package)` block. New mutator rows belong there, in its existing order.
* `Op::RemoveInstanceMethod` and `Op::HideInstanceMethod` are emitted by `rexx-classes`'s `build.rs`
  into the generated `setup_classes.rs`; read them from the generated file, do not re-extract from
  `Setup.cpp`.
* The deferral doc block naming `QueueClass`, `StemClass` and `VariableReference` is
  `crates/rexx-classes/src/native_classes.rs`, the `DEFERRALS` module comment. It states the missing
  mechanism as the reason. When the mechanism lands, that prose is false and must move with the code.
* Registry-level inherit already exists and already carries the native frame: `Interp::inherit_mixin`
  in `crates/rexx-exec/src/lib.rs` calls `self.classes().inherit(class, mixin)` and
  `self.blame_native_method(b"INHERIT", &scope)`. The message-level `~inherit` is a second caller of
  the same two, not a second implementation.

## 5. Ruling on the brief's `Queue`/`Stem` superclass rows

The brief says those rows do not close here and close at Task 23. That is its own claim about a state
this task can measure. Measure it rather than assuming it: record what `~superClasses` actually reads
after this task, and if it already agrees, say so and close the row here.
