/* Table C wiring row: provide.xml's class hierarchy list indents
   NumericComparator below Comparator, which claims Comparator is PRESENT IN
   NumericComparator's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .NumericComparator~id
say 'parent' .Comparator~id
say 'documented-edge' .NumericComparator~superClasses~hasItem(.Comparator)
say 'superclasses' .NumericComparator~superClasses~makeString('L', ' ')
