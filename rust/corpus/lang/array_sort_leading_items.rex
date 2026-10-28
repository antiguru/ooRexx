/* The sort family sorts the first ITEMS slots, refusing a hole among them. */
b = .array~of('c','a','b')
b~remove(3)
b~sort
say b~items b[1] b[2] b~size
cases = .array~of(".array~of('c','a','b')~~remove(2)", ".array~of('c','a','b')~~remove(1)", -
  ".array~new(5)~~put('z',1)~~put('y',2)~~put('x',5)", ".array~new(5)~~put('z',1)~~put('y',2)~~put('x',4)", -
  ".array~new(5)~~put('z',2)", ".array~new(5)~~put('z',1)", ".array~new(3)", ".array~new", -
  ".array~of('b','a')~~put('z', 10)", ".array~of('d','c','b','a')~~remove(4)~~remove(3)", -
  ".array~of('q','p')~~put('r',4)~~remove(2)")
do c over cases
  do m over .array~of('sort', 'stableSort', 'sortWith', 'stableSortWith')
    call one c, m
  end
end
exit
one:
  use arg c, m
  signal on syntax
  interpret 'a = ' c
  if m~caselessPos('with') > 0 then interpret 'a~'m'(.caselessComparator~new)'
  else interpret 'a~'m
  say c m '->' a~items a~size a~last a~toString('L', ',')
  return
syntax:
  say c m '->' condition('o')~code condition('o')~message
  return
