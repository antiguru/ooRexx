/* Table C method rows: List, class arm -- .List~hasMethod("M") for
   every method corpus/docs/class-methods.txt documents on this arm,
   one line per row and in the row set's own order. Derived by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'class' .List~hasMethod("new")
say 'class' .List~hasMethod("of")
