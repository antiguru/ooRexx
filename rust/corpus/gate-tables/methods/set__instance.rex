/* Table C method rows: Set, instance arm -- one line per method
   corpus/docs/class-methods.txt documents on this arm, asked of the
   instance `.Set~new` answers, in the row set's own order. That
   expression is corpus/docs/class-set.txt's committed construction
   program for this class, and carrying one is what `covered` claims.
   Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .Set~new
say 'instance' o~hasMethod("[]")
say 'instance' o~hasMethod("[]=")
say 'instance' o~hasMethod("allIndexes")
say 'instance' o~hasMethod("allItems")
say 'instance' o~hasMethod("at")
say 'instance' o~hasMethod("empty")
say 'instance' o~hasMethod("hasIndex")
say 'instance' o~hasMethod("hasItem")
say 'instance' o~hasMethod("index")
say 'instance' o~hasMethod("intersection")
say 'instance' o~hasMethod("isEmpty")
say 'instance' o~hasMethod("items")
say 'instance' o~hasMethod("makeArray")
say 'instance' o~hasMethod("put")
say 'instance' o~hasMethod("putAll")
say 'instance' o~hasMethod("remove")
say 'instance' o~hasMethod("removeItem")
say 'instance' o~hasMethod("subset")
say 'instance' o~hasMethod("supplier")
say 'instance' o~hasMethod("union")
say 'instance' o~hasMethod("xor")
say 'instance' o~hasMethod("difference")
say 'instance' o~hasMethod("disjoint")
say 'instance' o~hasMethod("equivalent")
