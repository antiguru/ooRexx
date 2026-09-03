/* Table C method rows: RexxContext, instance arm -- one line per method
   corpus/docs/class-methods.txt documents on this arm, asked of the
   instance `.context` answers, in the row set's own order. That
   expression is corpus/docs/class-set.txt's committed construction
   program for this class, and carrying one is what `covered` claims.
   Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .context
say 'instance' o~hasMethod("args")
say 'instance' o~hasMethod("condition")
say 'instance' o~hasMethod("digits")
say 'instance' o~hasMethod("executable")
say 'instance' o~hasMethod("form")
say 'instance' o~hasMethod("fuzz")
say 'instance' o~hasMethod("interpreter")
say 'instance' o~hasMethod("invocation")
say 'instance' o~hasMethod("line")
say 'instance' o~hasMethod("name")
say 'instance' o~hasMethod("package")
say 'instance' o~hasMethod("rs")
say 'instance' o~hasMethod("stackFrames")
say 'instance' o~hasMethod("thread")
say 'instance' o~hasMethod("variables")
