-- The Directory protocol on the condition object Message~errorCondition answers.
m = .t~new~start('boom')
m~wait
call SysSleep 0.3
c = m~errorCondition
signal on syntax name trapped
say 'items' c~items
say 'hasindex' c~hasIndex('CODE') c~hasEntry('CODE')
say 'allindexes' c~allIndexes~items c~allItems~items
say 'index' c~index(c~code)
say 'supplier' c~supplier~available c~makeArray~items c~isEmpty
c~setEntry('ZZ', 1)
say 'entry' c~entry('ZZ') c~remove('ZZ') c~hasItem(c~code)
say 'copy items' c~copy~items
c~empty
say 'empty' c~items
exit 0
trapped: say 'syntax' condition('O')~code
exit 0
::class t
::method boom unguarded
  return 1/0
