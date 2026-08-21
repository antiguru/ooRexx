/* Table C method rows: Relation, instance arm -- one line per method
   corpus/docs/class-methods.txt documents on this arm, asked of a bare
   ~new instance, in the row set's own order. Derived by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
o = .Relation~new
say 'instance' o~hasMethod("[]")
say 'instance' o~hasMethod("[]=")
say 'instance' o~hasMethod("allAt")
say 'instance' o~hasMethod("allIndex")
say 'instance' o~hasMethod("allIndexes")
say 'instance' o~hasMethod("allItems")
say 'instance' o~hasMethod("at")
say 'instance' o~hasMethod("difference")
say 'instance' o~hasMethod("empty")
say 'instance' o~hasMethod("hasIndex")
say 'instance' o~hasMethod("hasItem")
say 'instance' o~hasMethod("index")
say 'instance' o~hasMethod("intersection")
say 'instance' o~hasMethod("isEmpty")
say 'instance' o~hasMethod("items")
say 'instance' o~hasMethod("makeArray")
say 'instance' o~hasMethod("put")
say 'instance' o~hasMethod("remove")
say 'instance' o~hasMethod("removeAll")
say 'instance' o~hasMethod("removeItem")
say 'instance' o~hasMethod("subset")
say 'instance' o~hasMethod("supplier")
say 'instance' o~hasMethod("union")
say 'instance' o~hasMethod("uniqueIndexes")
say 'instance' o~hasMethod("xor")
say 'instance' o~hasMethod("disjoint")
say 'instance' o~hasMethod("equivalent")
say 'instance' o~hasMethod("putAll")
