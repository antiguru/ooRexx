/* Table C method rows: InputStream, instance arm -- one line per method
   corpus/docs/class-methods.txt documents on this arm, asked of a bare
   ~new instance, in the row set's own order. Derived by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
o = .InputStream~new
say 'instance' o~hasMethod("arrayIn")
say 'instance' o~hasMethod("charIn")
say 'instance' o~hasMethod("charOut")
say 'instance' o~hasMethod("chars")
say 'instance' o~hasMethod("close")
say 'instance' o~hasMethod("lineIn")
say 'instance' o~hasMethod("lineOut")
say 'instance' o~hasMethod("lines")
say 'instance' o~hasMethod("open")
say 'instance' o~hasMethod("position")
