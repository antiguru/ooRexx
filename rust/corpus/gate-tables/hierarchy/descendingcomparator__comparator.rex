/* Table C wiring row: provide.xml's class hierarchy list indents
   DescendingComparator below Comparator, which claims Comparator is PRESENT IN
   DescendingComparator's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .DescendingComparator~id
say 'parent' .Comparator~id
say 'documented-edge' .DescendingComparator~superClasses~hasItem(.Comparator)
say 'superclasses' .DescendingComparator~superClasses~makeString('L', ' ')
