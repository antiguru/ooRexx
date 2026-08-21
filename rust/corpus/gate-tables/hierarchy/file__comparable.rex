/* Table C wiring row: provide.xml's class hierarchy list indents
   File below Comparable, which claims Comparable is PRESENT IN
   File's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .File~id
say 'parent' .Comparable~id
say 'documented-edge' .File~superClasses~hasItem(.Comparable)
say 'superclasses' .File~superClasses~makeString('L', ' ')
