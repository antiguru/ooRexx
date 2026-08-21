/* Table C wiring row: provide.xml's class hierarchy list indents
   Directory below MapCollection, which claims MapCollection is PRESENT IN
   Directory's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .Directory~id
say 'parent' .MapCollection~id
say 'documented-edge' .Directory~superClasses~hasItem(.MapCollection)
say 'superclasses' .Directory~superClasses~makeString('L', ' ')
