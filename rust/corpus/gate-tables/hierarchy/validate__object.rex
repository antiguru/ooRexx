/* Table C wiring row: provide.xml's class hierarchy list indents
   Validate below Object, which claims Object is PRESENT IN
   Validate's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .Validate~id
say 'parent' .Object~id
say 'documented-edge' .Validate~superClasses~hasItem(.Object)
say 'superclasses' .Validate~superClasses~makeString('L', ' ')
