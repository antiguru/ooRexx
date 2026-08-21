/* Table C wiring row: provide.xml's class hierarchy list indents
   SetCollection below Collection, which claims Collection is PRESENT IN
   SetCollection's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .SetCollection~id
say 'parent' .Collection~id
say 'documented-edge' .SetCollection~superClasses~hasItem(.Collection)
say 'superclasses' .SetCollection~superClasses~makeString('L', ' ')
