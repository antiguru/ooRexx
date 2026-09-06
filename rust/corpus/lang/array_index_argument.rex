/* The index-taking half.  These go through validateIndex and raise a
   different error from the item-taking three in `array_item_argument.rex` --
   measured, and invisible to any zero-argument probe because both families
   are loud until the bodies exist. */

a = .Array~of('p', 'q')
call trapped 'REMOVE'
call trapped 'HASINDEX'
call trapped 'AT'
say 'all three raised SYNTAX'
v = a~hasIndex
say 'unreachable' v
exit

trapped: procedure expose a
  use arg name
  signal on syntax name oops
  v = a~send(name)
  say name 'did not raise'
  return
oops:
  say name 'raised' rc condition('C')
  return
