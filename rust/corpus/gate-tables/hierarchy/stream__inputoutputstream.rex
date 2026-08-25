/* Table C wiring row: provide.xml's class hierarchy list indents
   Stream below InputOutputStream, which claims InputOutputStream is PRESENT IN
   Stream's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .Stream~id
say 'parent' .InputOutputStream~id
supers = .Stream~superClasses
edge = 0
do at = 1 to supers~items
  if supers[at]~id == .InputOutputStream~id then edge = 1
end
say 'documented-edge' edge
say 'superclasses' .Stream~superClasses~makeString('L', ' ')
