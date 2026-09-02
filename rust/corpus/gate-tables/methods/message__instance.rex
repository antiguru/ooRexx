/* Table C method rows: Message, instance arm -- one line per method
   corpus/docs/class-methods.txt documents on this arm, asked of the
   instance `.Message~new(.Object~new, 'STRING')` answers, in the row set's own order. That
   expression is corpus/docs/class-set.txt's committed construction
   program for this class, and carrying one is what `covered` claims.
   Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .Message~new(.Object~new, 'STRING')
say 'instance' o~hasMethod("arguments")
say 'instance' o~hasMethod("completed")
say 'instance' o~hasMethod("errorCondition")
say 'instance' o~hasMethod("halt")
say 'instance' o~hasMethod("hasError")
say 'instance' o~hasMethod("hasResult")
say 'instance' o~hasMethod("messageComplete")
say 'instance' o~hasMethod("messageName")
say 'instance' o~hasMethod("notify")
say 'instance' o~hasMethod("reply")
say 'instance' o~hasMethod("replyWith")
say 'instance' o~hasMethod("result")
say 'instance' o~hasMethod("send")
say 'instance' o~hasMethod("sendWith")
say 'instance' o~hasMethod("start")
say 'instance' o~hasMethod("startWith")
say 'instance' o~hasMethod("target")
say 'instance' o~hasMethod("triggered")
say 'instance' o~hasMethod("wait")
say 'instance' o~hasMethod("cancel")
