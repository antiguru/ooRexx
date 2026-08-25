/* Table C wiring row: provide.xml's class hierarchy list indents
   Stem below MapCollection, which claims MapCollection is PRESENT IN
   Stem's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .Stem~id
say 'parent' .MapCollection~id
supers = .Stem~superClasses
edge = 0
do at = 1 to supers~items
  if supers[at]~id == .MapCollection~id then edge = 1
end
say 'documented-edge' edge
say 'superclasses' .Stem~superClasses~makeString('L', ' ')
