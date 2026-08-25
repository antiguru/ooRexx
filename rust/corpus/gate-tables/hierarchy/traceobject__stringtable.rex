/* Table C wiring row: provide.xml's class hierarchy list indents
   TraceObject below StringTable, which claims StringTable is PRESENT IN
   TraceObject's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .TraceObject~id
say 'parent' .StringTable~id
supers = .TraceObject~superClasses
edge = 0
do at = 1 to supers~items
  if supers[at]~id == .StringTable~id then edge = 1
end
say 'documented-edge' edge
say 'superclasses' .TraceObject~superClasses~makeString('L', ' ')
