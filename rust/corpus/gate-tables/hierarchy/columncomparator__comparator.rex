/* Table C wiring row: provide.xml's class hierarchy list indents
   ColumnComparator below Comparator, which claims Comparator is PRESENT IN
   ColumnComparator's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .ColumnComparator~id
say 'parent' .Comparator~id
say 'documented-edge' .ColumnComparator~superClasses~hasItem(.Comparator)
say 'superclasses' .ColumnComparator~superClasses~makeString('L', ' ')
