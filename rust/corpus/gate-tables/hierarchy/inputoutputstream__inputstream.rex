/* Table C wiring row: provide.xml's class hierarchy list indents
   InputOutputStream below InputStream, which claims InputStream is PRESENT IN
   InputOutputStream's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .InputOutputStream~id
say 'parent' .InputStream~id
supers = .InputOutputStream~superClasses
edge = 0
do at = 1 to supers~items
  if supers[at]~id == .InputStream~id then edge = 1
end
say 'documented-edge' edge
say 'superclasses' .InputOutputStream~superClasses~makeString('L', ' ')
