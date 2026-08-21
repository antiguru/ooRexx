/* Table C wiring row: provide.xml's class hierarchy list indents
   Set below MapCollection, which claims MapCollection is PRESENT IN
   Set's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .Set~id
say 'parent' .MapCollection~id
say 'documented-edge' .Set~superClasses~hasItem(.MapCollection)
say 'superclasses' .Set~superClasses~makeString('L', ' ')
