/* Object~instanceMethods lists a class object's own setMethod entries, as
   it lists an instance's and as instanceMethod finds them. */
s = .sub~new
s~sm('OWN', 'return 2')
say 'instance' s~instanceMethod('OWN')~source~makeString cnt(s~instanceMethods, 'OWN') cnt(s~instanceMethods(.nil), 'OWN')
.sub~sm('OWN', 'return 3')
say 'class' .sub~instanceMethod('OWN')~source~makeString cnt(.sub~instanceMethods, 'OWN') cnt(.sub~instanceMethods(.nil), 'OWN')
say 'class item' (find(.sub~instanceMethods(.nil), 'OWN') == .sub~instanceMethod('OWN'))
exit

cnt: procedure
  use arg s, name
  n = 0
  do while s~available
    if s~index == name then n = n + 1
    s~next
  end
  return n

find: procedure
  use arg s, name
  do while s~available
    if s~index == name then return s~item
    s~next
  end
  return .nil

::class sub
::method sm class
  forward message 'SETMETHOD'
::method sm
  forward message 'SETMETHOD'
