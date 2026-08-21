/* Table C wiring row: provide.xml's class hierarchy list indents
   Buffer below Object, which claims Object is PRESENT IN
   Buffer's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .Buffer~id
say 'parent' .Object~id
say 'documented-edge' .Buffer~superClasses~hasItem(.Object)
say 'superclasses' .Buffer~superClasses~makeString('L', ' ')
