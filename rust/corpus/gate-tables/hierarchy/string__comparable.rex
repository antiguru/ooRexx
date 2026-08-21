/* Table C wiring row: provide.xml's class hierarchy list indents
   String below Comparable, which claims Comparable is PRESENT IN
   String's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .String~id
say 'parent' .Comparable~id
say 'documented-edge' .String~superClasses~hasItem(.Comparable)
say 'superclasses' .String~superClasses~makeString('L', ' ')
