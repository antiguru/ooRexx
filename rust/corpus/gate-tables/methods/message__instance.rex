/* Table C method rows: Message, instance arm. corpus/docs/class-set.txt
   records this class as `not-covered`, because
   no construction program is committed; a bare ~new raises 93.901 on the oracle.
   So ~new raises and no line below it is reached; the row's evidence
   is that raise, which is what the row set says there is to have.
   Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .Message~new
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
