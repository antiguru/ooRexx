/* Table C method rows: Stream, instance arm -- one line per method
   corpus/docs/class-methods.txt documents on this arm, asked of the
   instance `.Stream~new('/nonexistent-gate-table-c-stream')` answers, in the row set's own order. That
   expression is corpus/docs/class-set.txt's committed construction
   program for this class, and carrying one is what `covered` claims.
   Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .Stream~new('/nonexistent-gate-table-c-stream')
say 'instance' o~hasMethod("arrayIn")
say 'instance' o~hasMethod("arrayOut")
say 'instance' o~hasMethod("charIn")
say 'instance' o~hasMethod("charOut")
say 'instance' o~hasMethod("chars")
say 'instance' o~hasMethod("close")
say 'instance' o~hasMethod("command")
say 'instance' o~hasMethod("description")
say 'instance' o~hasMethod("flush")
say 'instance' o~hasMethod("init")
say 'instance' o~hasMethod("lineIn")
say 'instance' o~hasMethod("lineOut")
say 'instance' o~hasMethod("lines")
say 'instance' o~hasMethod("makeArray")
say 'instance' o~hasMethod("open")
say 'instance' o~hasMethod("position")
say 'instance' o~hasMethod("qualify")
say 'instance' o~hasMethod("query")
say 'instance' o~hasMethod("say")
say 'instance' o~hasMethod("seek")
say 'instance' o~hasMethod("state")
say 'instance' o~hasMethod("string")
say 'instance' o~hasMethod("supplier")
say 'instance' o~hasMethod("uninit")
