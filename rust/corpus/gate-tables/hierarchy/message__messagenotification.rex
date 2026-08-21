/* Table C wiring row: provide.xml's class hierarchy list indents
   Message below MessageNotification, which claims MessageNotification is PRESENT IN
   Message's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .Message~id
say 'parent' .MessageNotification~id
say 'documented-edge' .Message~superClasses~hasItem(.MessageNotification)
say 'superclasses' .Message~superClasses~makeString('L', ' ')
