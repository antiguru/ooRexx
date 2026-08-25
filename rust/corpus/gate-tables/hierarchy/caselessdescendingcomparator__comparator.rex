/* Table C wiring row: provide.xml's class hierarchy list indents
   CaselessDescendingComparator below Comparator, which claims Comparator is PRESENT IN
   CaselessDescendingComparator's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .CaselessDescendingComparator~id
say 'parent' .Comparator~id
supers = .CaselessDescendingComparator~superClasses
edge = 0
do at = 1 to supers~items
  if supers[at]~id == .Comparator~id then edge = 1
end
say 'documented-edge' edge
say 'superclasses' .CaselessDescendingComparator~superClasses~makeString('L', ' ')
