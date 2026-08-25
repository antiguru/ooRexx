/* Table C wiring row: provide.xml's class hierarchy list indents
   OrderedCollection below Collection, which claims Collection is PRESENT IN
   OrderedCollection's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .OrderedCollection~id
say 'parent' .Collection~id
supers = .OrderedCollection~superClasses
edge = 0
do at = 1 to supers~items
  if supers[at]~id == .Collection~id then edge = 1
end
say 'documented-edge' edge
say 'superclasses' .OrderedCollection~superClasses~makeString('L', ' ')
