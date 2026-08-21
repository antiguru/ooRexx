/* Table C wiring row: provide.xml's class hierarchy list indents
   CaselessColumnComparator below Comparator, which claims Comparator is PRESENT IN
   CaselessColumnComparator's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .CaselessColumnComparator~id
say 'parent' .Comparator~id
say 'documented-edge' .CaselessColumnComparator~superClasses~hasItem(.Comparator)
say 'superclasses' .CaselessColumnComparator~superClasses~makeString('L', ' ')
