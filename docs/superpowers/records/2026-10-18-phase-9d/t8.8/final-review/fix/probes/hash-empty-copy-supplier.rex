do cls over .array~of(.Table, .IdentityTable, .Relation, .Bag, .Directory, .StringTable)
  t = cls~new
  t~put('one', 'one')
  t~put('two', 'two')
  c = t~copy
  supplier = t~supplier
  t~empty
  t~put('new', 'new')
  snapshot = .array~new
  do while supplier~available
    snapshot~append(supplier~item)
    supplier~next
  end
  snapshot~sort
  say cls~id t~items c~items c~hasItem('one') c~hasItem('two') snapshot~makeString('L', ',')
  c~empty
  say 'reused' t~hasItem('new') c~items c~isEmpty
end
