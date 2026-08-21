/* Table C wiring row: provide.xml's class hierarchy list indents
   CaselessDescendingComparator below Comparator, which claims Comparator is PRESENT IN
   CaselessDescendingComparator's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .CaselessDescendingComparator~id
say 'parent' .Comparator~id
say 'documented-edge' .CaselessDescendingComparator~superClasses~hasItem(.Comparator)
say 'superclasses' .CaselessDescendingComparator~superClasses~makeString('L', ' ')
