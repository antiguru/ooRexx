/* Table C wiring row: provide.xml's class hierarchy list indents
   Array below OrderedCollection, which claims OrderedCollection is PRESENT IN
   Array's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .Array~id
say 'parent' .OrderedCollection~id
supers = .Array~superClasses
edge = 0
do at = 1 to supers~items
  if supers[at]~id == .OrderedCollection~id then edge = 1
end
say 'documented-edge' edge
say 'superclasses' .Array~superClasses~makeString('L', ' ')
