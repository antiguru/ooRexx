/* Table C wiring row: provide.xml's class hierarchy list indents
   TimeSpan below Comparable, which claims Comparable is PRESENT IN
   TimeSpan's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .TimeSpan~id
say 'parent' .Comparable~id
supers = .TimeSpan~superClasses
edge = 0
do at = 1 to supers~items
  if supers[at]~id == .Comparable~id then edge = 1
end
say 'documented-edge' edge
say 'superclasses' .TimeSpan~superClasses~makeString('L', ' ')
