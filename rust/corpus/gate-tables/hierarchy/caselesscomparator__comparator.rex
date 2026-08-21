/* Table C wiring row: provide.xml's class hierarchy list indents
   CaselessComparator below Comparator, which claims Comparator is PRESENT IN
   CaselessComparator's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .CaselessComparator~id
say 'parent' .Comparator~id
say 'documented-edge' .CaselessComparator~superClasses~hasItem(.Comparator)
say 'superclasses' .CaselessComparator~superClasses~makeString('L', ' ')
