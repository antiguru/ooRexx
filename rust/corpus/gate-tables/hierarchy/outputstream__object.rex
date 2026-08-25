/* Table C wiring row: provide.xml's class hierarchy list indents
   OutputStream below Object, which claims Object is PRESENT IN
   OutputStream's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .OutputStream~id
say 'parent' .Object~id
supers = .OutputStream~superClasses
edge = 0
do at = 1 to supers~items
  if supers[at]~id == .Object~id then edge = 1
end
say 'documented-edge' edge
say 'superclasses' .OutputStream~superClasses~makeString('L', ' ')
