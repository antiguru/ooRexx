/* Table C wiring row: provide.xml's class hierarchy list indents
   Stream below InputOutputStream, which claims InputOutputStream is PRESENT IN
   Stream's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .Stream~id
say 'parent' .InputOutputStream~id
say 'documented-edge' .Stream~superClasses~hasItem(.InputOutputStream)
say 'superclasses' .Stream~superClasses~makeString('L', ' ')
