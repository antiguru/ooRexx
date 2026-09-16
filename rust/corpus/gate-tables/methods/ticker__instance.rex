/* Table C method rows: Ticker, instance arm -- one line per method
   corpus/docs/class-methods.txt documents on this arm, asked of the
   instance `.Ticker~new(99999, .Message~new(.Object~new, 'STRING'))~~cancel` answers, in the row set's own order. That
   expression is corpus/docs/class-set.txt's committed construction
   program for this class, and carrying one is what `covered` claims.
   Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .Ticker~new(99999, .Message~new(.Object~new, 'STRING'))~~cancel
say 'instance' o~hasMethod("attachment")
say 'instance' o~hasMethod("cancel")
say 'instance' o~hasMethod("canceled")
say 'instance' o~hasMethod("cancelled")
say 'instance' o~hasMethod("init")
say 'instance' o~hasMethod("interval")
