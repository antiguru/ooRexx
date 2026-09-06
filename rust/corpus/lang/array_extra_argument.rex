/* A zero-argument method given one.  A third error again, so the shared
   surface raises three different numbers for three different kinds of bad
   call. */

a = .Array~of('p', 'q')
call trapped 'ALLITEMS'
call trapped 'ALLINDEXES'
call trapped 'ISEMPTY'
call trapped 'SUPPLIER'
say 'all four raised SYNTAX'
v = a~allItems('unwanted')
say 'unreachable' v
exit

trapped: procedure expose a
  use arg name
  signal on syntax name oops
  v = a~send(name, 'unwanted')
  say name 'did not raise'
  return
oops:
  say name 'raised' rc condition('C')
  return
