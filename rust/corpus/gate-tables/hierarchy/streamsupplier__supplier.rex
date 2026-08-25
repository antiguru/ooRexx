/* Table C wiring row: provide.xml's class hierarchy list indents
   StreamSupplier below Supplier, which claims Supplier is PRESENT IN
   StreamSupplier's ~superClasses and never that it is the whole of it.
   Derived from corpus/docs/hierarchy-edges.txt by
   crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on
   every run and compares it in both directions. */
say 'child' .StreamSupplier~id
say 'parent' .Supplier~id
supers = .StreamSupplier~superClasses
edge = 0
do at = 1 to supers~items
  if supers[at]~id == .Supplier~id then edge = 1
end
say 'documented-edge' edge
say 'superclasses' .StreamSupplier~superClasses~makeString('L', ' ')
