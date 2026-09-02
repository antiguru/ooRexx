/* Table C method rows: Supplier, instance arm -- one line per method
   corpus/docs/class-methods.txt documents on this arm, asked of the
   instance `.Supplier~new(.Array~new, .Array~new)` answers, in the row set's own order. That
   expression is corpus/docs/class-set.txt's committed construction
   program for this class, and carrying one is what `covered` claims.
   Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .Supplier~new(.Array~new, .Array~new)
say 'instance' o~hasMethod("allIndexes")
say 'instance' o~hasMethod("allItems")
say 'instance' o~hasMethod("available")
say 'instance' o~hasMethod("index")
say 'instance' o~hasMethod("item")
say 'instance' o~hasMethod("next")
say 'instance' o~hasMethod("supplier")
