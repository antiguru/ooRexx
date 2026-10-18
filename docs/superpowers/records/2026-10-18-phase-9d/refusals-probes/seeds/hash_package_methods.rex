-- The Directory protocol on a package's .methods table.
signal on syntax name trapped
t = .methods
say 'class' t~class~id
say 'items' t~items t~hasIndex('M') t~allIndexes~items t~allItems~items
say 'supplier' t~supplier~available t~makeArray~items t~isEmpty
say 'index' (t~index(t['M']) == 'M') t~hasItem(t['M'])
say 'copy' t~copy~items
t~remove('M')
say 'removed' t~items
exit 0
trapped: say 'syntax' condition('O')~code
exit 0
::method m
  return 1
