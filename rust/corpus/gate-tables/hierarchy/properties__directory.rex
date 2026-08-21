/* Table C wiring row: provide.xml's class hierarchy list indents
   Properties below Directory, which claims Directory is PRESENT IN
   Properties's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .Properties~id
say 'parent' .Directory~id
say 'documented-edge' .Properties~superClasses~hasItem(.Directory)
say 'superclasses' .Properties~superClasses~makeString('L', ' ')
