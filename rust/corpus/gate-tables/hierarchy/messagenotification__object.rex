/* Table C wiring row: provide.xml's class hierarchy list indents
   MessageNotification below Object, which claims Object is PRESENT IN
   MessageNotification's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .MessageNotification~id
say 'parent' .Object~id
supers = .MessageNotification~superClasses
edge = 0
do at = 1 to supers~items
  if supers[at]~id == .Object~id then edge = 1
end
say 'documented-edge' edge
say 'superclasses' .MessageNotification~superClasses~makeString('L', ' ')
