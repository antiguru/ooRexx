/* Table C method rows: Monitor, instance arm -- one line per method
   corpus/docs/class-methods.txt documents on this arm, asked of a bare
   ~new instance, in the row set's own order. Derived by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
o = .Monitor~new
say 'instance' o~hasMethod("current")
say 'instance' o~hasMethod("destination")
say 'instance' o~hasMethod("init")
say 'instance' o~hasMethod("unknown")
