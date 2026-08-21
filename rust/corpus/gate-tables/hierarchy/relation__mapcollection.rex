/* Table C wiring row: provide.xml's class hierarchy list indents
   Relation below MapCollection, which claims MapCollection is PRESENT IN
   Relation's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .Relation~id
say 'parent' .MapCollection~id
say 'documented-edge' .Relation~superClasses~hasItem(.MapCollection)
say 'superclasses' .Relation~superClasses~makeString('L', ' ')
