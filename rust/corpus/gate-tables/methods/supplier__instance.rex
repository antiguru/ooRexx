/* Table C method rows: Supplier, instance arm. corpus/docs/class-set.txt
   records this class as `not-covered`, because
   no construction program is committed; a bare ~new raises 93.903 on the oracle.
   So ~new raises and no line below it is reached; the row's evidence
   is that raise, which is what the row set says there is to have.
   Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .Supplier~new
say 'instance' o~hasMethod("allIndexes")
say 'instance' o~hasMethod("allItems")
say 'instance' o~hasMethod("available")
say 'instance' o~hasMethod("index")
say 'instance' o~hasMethod("item")
say 'instance' o~hasMethod("next")
say 'instance' o~hasMethod("supplier")
