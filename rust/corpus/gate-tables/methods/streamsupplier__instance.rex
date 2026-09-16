/* Table C method rows: StreamSupplier, instance arm -- one line per method
   corpus/docs/class-methods.txt documents on this arm, asked of the
   instance `.Stream~new('/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/corpus/gate-tables/fixtures/streamsupplier_seed.txt')~supplier` answers, in the row set's own order. That
   expression is corpus/docs/class-set.txt's committed construction
   program for this class, and carrying one is what `covered` claims.
   Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .Stream~new('/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/corpus/gate-tables/fixtures/streamsupplier_seed.txt')~supplier
say 'instance' o~hasMethod("available")
say 'instance' o~hasMethod("index")
say 'instance' o~hasMethod("init")
say 'instance' o~hasMethod("item")
say 'instance' o~hasMethod("next")
say 'instance' o~hasMethod("allIndexes")
say 'instance' o~hasMethod("allItems")
say 'instance' o~hasMethod("supplier")
