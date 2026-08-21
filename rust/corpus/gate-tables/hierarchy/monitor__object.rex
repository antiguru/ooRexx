/* Table C wiring row: provide.xml's class hierarchy list indents
   Monitor below Object, which claims Object is PRESENT IN
   Monitor's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .Monitor~id
say 'parent' .Object~id
say 'documented-edge' .Monitor~superClasses~hasItem(.Object)
say 'superclasses' .Monitor~superClasses~makeString('L', ' ')
