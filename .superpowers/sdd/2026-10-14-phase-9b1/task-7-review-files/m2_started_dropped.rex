o = .k~new
a = o~start('SLOW', 1)
a = .nil
do 10; z = copies('x',100000); end
b = .Message~new(o, 'FAST')
say 'b completed before send:' b~completed
say 'b result:' b~send b~completed
c = o~start('SLOW', 2)
d = o~start('FAST')
say 'd' d~result
say 'c' c~result
say 'main done'
::class k
::method slow unguarded
  use arg n
  do 20; z = copies('y',100000); end
  say 'slow' n 'done'
  return 7
::method fast unguarded
  return 8
