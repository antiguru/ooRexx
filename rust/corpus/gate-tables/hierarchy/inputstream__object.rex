/* Table C wiring row: provide.xml's class hierarchy list indents
   InputStream below Object, which claims Object is PRESENT IN
   InputStream's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .InputStream~id
say 'parent' .Object~id
say 'documented-edge' .InputStream~superClasses~hasItem(.Object)
say 'superclasses' .InputStream~superClasses~makeString('L', ' ')
