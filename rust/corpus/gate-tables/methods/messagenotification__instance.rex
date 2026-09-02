/* Table C method rows: MessageNotification, instance arm -- one line per method
   corpus/docs/class-methods.txt documents on this arm, asked of the
   instance `.MessageNotification~new` answers, in the row set's own order. That
   expression is corpus/docs/class-set.txt's committed construction
   program for this class, and carrying one is what `covered` claims.
   Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .MessageNotification~new
say 'instance' o~hasMethod("messageComplete")
