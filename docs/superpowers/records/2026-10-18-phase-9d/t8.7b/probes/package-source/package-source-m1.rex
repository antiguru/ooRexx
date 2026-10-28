say .lib~method('M')~source~items
say .context~package~findroutine('PUB')~source~items
say .context~package~source~items > 0
say .context~package~findpublicclass('LIB')~method('M')~source~items
say .array~method('SORT')~source~items
say .orderedcollection~method('SORTWITH')~source~items
say .stream~method('LINEIN')~source~items
say .stream~method('INIT')~source~items
say .datetime~method('INIT')~source~items
say .method~new('x', 'say 1')~source~items
say .routine~new('x', 'say 1; say 2')~source~items
say .stream~method('INIT')~package~name~length > 0
say .stream~method('INIT')~package~source~items
say .stream~method('INIT')~package~sourceline(1)'|'
say .stream~method('INIT')~package~sourcesize
exit
::requires 'req.cls'
::method m
  return 1
