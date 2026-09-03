/* Table C method rows: Routine, instance arm -- one line per method
   corpus/docs/class-methods.txt documents on this arm, asked of the
   instance `.routines~r` answers, in the row set's own order. That
   expression is corpus/docs/class-set.txt's committed construction
   program for this class, and carrying one is what `covered` claims.
   `::routine r` below the readbacks is that row's `directives` field,
   which is what the expression reads.
   Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .routines~r
say 'instance' o~hasMethod("[]")
say 'instance' o~hasMethod("annotation")
say 'instance' o~hasMethod("annotations")
say 'instance' o~hasMethod("call")
say 'instance' o~hasMethod("callWith")
say 'instance' o~hasMethod("package")
say 'instance' o~hasMethod("setSecurityManager")
say 'instance' o~hasMethod("source")
::routine r
