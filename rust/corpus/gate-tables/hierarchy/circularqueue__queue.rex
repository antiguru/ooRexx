/* Table C wiring row: provide.xml's class hierarchy list indents
   CircularQueue below Queue, which claims Queue is PRESENT IN
   CircularQueue's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .CircularQueue~id
say 'parent' .Queue~id
supers = .CircularQueue~superClasses
edge = 0
do at = 1 to supers~items
  if supers[at]~id == .Queue~id then edge = 1
end
say 'documented-edge' edge
say 'superclasses' .CircularQueue~superClasses~makeString('L', ' ')
