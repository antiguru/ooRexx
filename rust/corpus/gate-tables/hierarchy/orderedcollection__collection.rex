/* Table C wiring row: provide.xml's class hierarchy list indents
   OrderedCollection below Collection, which claims Collection is PRESENT IN
   OrderedCollection's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .OrderedCollection~id
say 'parent' .Collection~id
say 'documented-edge' .OrderedCollection~superClasses~hasItem(.Collection)
say 'superclasses' .OrderedCollection~superClasses~makeString('L', ' ')
