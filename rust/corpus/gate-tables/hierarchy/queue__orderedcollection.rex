/* Table C wiring row: provide.xml's class hierarchy list indents
   Queue below OrderedCollection, which claims OrderedCollection is PRESENT IN
   Queue's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .Queue~id
say 'parent' .OrderedCollection~id
say 'documented-edge' .Queue~superClasses~hasItem(.OrderedCollection)
say 'superclasses' .Queue~superClasses~makeString('L', ' ')
