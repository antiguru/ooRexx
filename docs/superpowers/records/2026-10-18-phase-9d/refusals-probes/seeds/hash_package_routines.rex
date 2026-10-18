-- The StringTable protocol on a package's .routines table and its
-- publicRoutines.
signal on syntax name trapped
t = .routines
say 'class' t~class~id
say 'items' t~items t~hasIndex('R') t~allIndexes~items t~allItems~items
say 'supplier' t~supplier~available t~makeArray~items t~isEmpty
say 'copy' t~copy~items
p = .context~package~publicRoutines
say 'public' p~class~id p~items p~hasIndex('R')
exit 0
trapped: say 'syntax' condition('O')~code
exit 0
::routine r public
  return 1
