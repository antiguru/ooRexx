b = .array~of('c','a','b')
b~remove(3)
say b~size b~items b~last b~dimension(1)
b~sort
say b~items b[1] b[2] b~size
call try ".array~of('c','a','b')~~remove(2)"
call try ".array~of('c','a','b')~~remove(1)"
call try ".array~new(5)~~put('z',1)~~put('y',2)~~put('x',5)"
call try ".array~new(5)~~put('z',1)~~put('y',2)~~put('x',4)"
call try ".array~new(5)~~put('z',2)"
call try ".array~new(3)"
call try ".array~new"
call try ".array~of('b','a')~~put('z', 10)"
exit
try:
  signal on syntax
  interpret 'a = ' arg(1)
  do m over .array~of('sort', 'stableSort', 'sortWith', 'stableSortWith')
    b = a~copy
    if m~caselessAbbrev('sortw', 5) | m~caselessPos('with') > 0 then interpret 'b~'m'(.caselessComparator~new)'
    else interpret 'b~'m
    say arg(1) m '->' b~items b~size b~toString('L', ',')
  end
  return
syntax:
  say arg(1) m '->' condition('o')~code condition('o')~message
  return
