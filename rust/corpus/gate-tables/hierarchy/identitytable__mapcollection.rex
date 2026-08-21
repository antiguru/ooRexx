/* Table C wiring row: provide.xml's class hierarchy list indents
   IdentityTable below MapCollection, which claims MapCollection is PRESENT IN
   IdentityTable's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .IdentityTable~id
say 'parent' .MapCollection~id
say 'documented-edge' .IdentityTable~superClasses~hasItem(.MapCollection)
say 'superclasses' .IdentityTable~superClasses~makeString('L', ' ')
