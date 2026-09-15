/* A Directory's index reads name its setMethod entries without running them:
 * allIndexes, makeArray, DO OVER, items and hasIndex read the method table's
 * indexes, while allItems, supplier and [] run the method, which says so here
 * each time it runs.  The same holds on .local.
 */

d = .Directory~new
d['plain'] = 1
d~setMethod('TICK', 'say "  ran TICK"; return 2')
call enumerate 'directory', d

.local~setMethod('ZZTICK', 'say "  ran ZZTICK"; return 3')
.local['STDQUE'] = 'queue stand-in'
call enumerate 'local', .local
exit

enumerate:
  use arg label, dir
  say label 'allIndexes'
  say ' ' dir~allIndexes~lastItem
  say label 'makeArray'
  say ' ' dir~makeArray~lastItem
  say label 'do over'
  n = 0
  do name over dir
    n = n + 1
  end
  say ' ' n
  say label 'items and hasIndex'
  say ' ' dir~items dir~hasIndex(dir~allIndexes~lastItem)
  say label 'allItems'
  say ' ' dir~allItems~lastItem
  say label 'supplier'
  s = dir~supplier
  do while s~available
    last = s~item
    s~next
  end
  say ' ' last
  say label 'at'
  say ' ' dir[dir~allIndexes~lastItem]
  return
