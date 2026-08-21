/* Table C wiring row: provide.xml's class hierarchy list indents
   WeakReference below Object, which claims Object is PRESENT IN
   WeakReference's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .WeakReference~id
say 'parent' .Object~id
say 'documented-edge' .WeakReference~superClasses~hasItem(.Object)
say 'superclasses' .WeakReference~superClasses~makeString('L', ' ')
