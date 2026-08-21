/* Table C wiring row: provide.xml's class hierarchy list indents
   CircularQueue below Queue, which claims Queue is PRESENT IN
   CircularQueue's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .CircularQueue~id
say 'parent' .Queue~id
say 'documented-edge' .CircularQueue~superClasses~hasItem(.Queue)
say 'superclasses' .CircularQueue~superClasses~makeString('L', ' ')
