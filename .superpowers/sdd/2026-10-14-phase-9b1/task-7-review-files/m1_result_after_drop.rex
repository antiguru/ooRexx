o = .array~of(1,2)
m = .Message~new(o,'ITEMS'); r = m~send
r2 = m~result
m = .nil
do 30; z = copies('x',100000); end
say 'a' r r2
m = .Message~new(o,'ITEMS'); m~start; r3 = m~result; m = .nil
do 30; z = copies('x',100000); end
say 'b' r3
/* fresh messages in recycled slots must not inherit an old outcome */
do i = 1 to 20
  n = .Message~new(o,'ITEMS')
  if n~completed then say 'stale completed at' i
  if i // 2 = 0 then n~send
  else do; n~start; x = n~result; end
  if \n~completed then say 'not completed at' i
  call tryboom i
  z = copies('x',100000)
end
say 'c done'
exit
tryboom: procedure
  w = .Message~new(.k~new,'BOOM')
  w~start
  signal on syntax name s1
  y = w~result
  say 'no raise at' arg(1)
  return
s1:
  if w~hasError then return
  say 'raise without hasError at' arg(1)
  return
::class k
::method boom
  raise syntax 93.900 array('boom')
