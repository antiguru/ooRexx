/* Table C method rows: Comparable, instance arm -- one line per method
   corpus/docs/class-methods.txt documents on this arm, asked of the
   instance `.Comparable~new` answers, in the row set's own order. That
   expression is corpus/docs/class-set.txt's committed construction
   program for this class, and carrying one is what `covered` claims.
   Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .Comparable~new
say 'instance' o~hasMethod("compareTo")
