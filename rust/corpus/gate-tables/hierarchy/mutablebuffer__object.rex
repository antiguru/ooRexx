/* Table C wiring row: provide.xml's class hierarchy list indents
   MutableBuffer below Object, which claims Object is PRESENT IN
   MutableBuffer's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .MutableBuffer~id
say 'parent' .Object~id
say 'documented-edge' .MutableBuffer~superClasses~hasItem(.Object)
say 'superclasses' .MutableBuffer~superClasses~makeString('L', ' ')
