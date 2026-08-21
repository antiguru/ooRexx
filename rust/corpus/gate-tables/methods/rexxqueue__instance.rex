/* Table C method rows: RexxQueue, instance arm -- one line per method
   corpus/docs/class-methods.txt documents on this arm, asked of a bare
   ~new instance, in the row set's own order. Derived by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
o = .RexxQueue~new
say 'instance' o~hasMethod("delete")
say 'instance' o~hasMethod("empty")
say 'instance' o~hasMethod("get")
say 'instance' o~hasMethod("init")
say 'instance' o~hasMethod("lineIn")
say 'instance' o~hasMethod("lineOut")
say 'instance' o~hasMethod("makeArray")
say 'instance' o~hasMethod("pull")
say 'instance' o~hasMethod("push")
say 'instance' o~hasMethod("queue")
say 'instance' o~hasMethod("queued")
say 'instance' o~hasMethod("say")
say 'instance' o~hasMethod("set")
