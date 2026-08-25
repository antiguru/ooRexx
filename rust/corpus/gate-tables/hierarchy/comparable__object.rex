/* Table C wiring row: provide.xml's class hierarchy list indents
   Comparable below Object, which claims Object is PRESENT IN
   Comparable's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .Comparable~id
say 'parent' .Object~id
supers = .Comparable~superClasses
edge = 0
do at = 1 to supers~items
  if supers[at]~id == .Object~id then edge = 1
end
say 'documented-edge' edge
say 'superclasses' .Comparable~superClasses~makeString('L', ' ')
