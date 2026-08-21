/* Table C method rows: EventSemaphore, instance arm -- one line per method
   corpus/docs/class-methods.txt documents on this arm, asked of a bare
   ~new instance, in the row set's own order. Derived by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
o = .EventSemaphore~new
say 'instance' o~hasMethod("isPosted")
say 'instance' o~hasMethod("post")
say 'instance' o~hasMethod("reset")
say 'instance' o~hasMethod("uninit")
say 'instance' o~hasMethod("wait")
