/* SysStemSort: order, type, the element and column ranges, the stem forms the
   first argument takes, the stem shapes that are not plain arrays, and every
   error the routine raises. A raise from inside the sort itself is trapped
   in each case: untrapped it crashes the oracle (oracle-crashes.txt entry 37). */
call t "a.0 = 3; a.1 = 'b'; a.2 = 'c'; a.3 = 'a'", "call SysStemSort 'a.'"
call t "a.0 = 3; a.1 = 'b'; a.2 = 'c'; a.3 = 'a'", "call SysStemSort a., 'Descending'"
call t "a.0 = 3; a.1 = 'b'; a.2 = 'c'; a.3 = 'a'", "call SysStemSort 'a'"
call t "a.0 = 4; a.1 = 'b'; a.2 = 'C'; a.3 = 'a'; a.4 = 'D'", "call SysStemSort 'a.', 'A', 'C'"
call t "a.0 = 4; a.1 = 'b'; a.2 = 'C'; a.3 = 'a'; a.4 = 'D'", "call SysStemSort 'a.', 'a', 'Ignore'"
call t "a.0 = 4; a.1 = 'b'; a.2 = 'C'; a.3 = 'a'; a.4 = 'D'", "call SysStemSort 'a.', 'd', 'i'"
call t "a.0 = 4; a.1 = 'x1'; a.2 = 'X2'; a.3 = 'x3'; a.4 = 'X0'", "call SysStemSort 'a.', , 'I', , , 1, 1"
call t "a.0 = 4; a.1 = 'x1'; a.2 = 'X2'; a.3 = 'x3'; a.4 = 'X0'", "call SysStemSort 'a.', 'D', 'I', , , 1, 1"
call t "a.0 = 5; a.1 = 'e'; a.2 = 'd'; a.3 = 'c'; a.4 = 'b'; a.5 = 'a'", "call SysStemSort 'a.', , , 2, 4"
call t "a.0 = 5; a.1 = 'e'; a.2 = 'd'; a.3 = 'c'; a.4 = 'b'; a.5 = 'a'", "call SysStemSort 'a.', , , 3"
call t "a.0 = 5; a.1 = 'e'; a.2 = 'd'; a.3 = 'c'; a.4 = 'b'; a.5 = 'a'", "call SysStemSort 'a.', , , , 2"
call t "a.0 = 3; a.1 = 'b'; a.2 = 'c'; a.3 = 'a'", "call SysStemSort 'a.', , , 1, 999999999999999999"
call t "a.0 = 4; a.1 = 'zb'; a.2 = 'ya'; a.3 = 'x'; a.4 = 'yab'", "call SysStemSort 'a.', , , , , 2"
call t "a.0 = 4; a.1 = 'zb'; a.2 = 'ya'; a.3 = 'x'; a.4 = 'yab'", "call SysStemSort 'a.', , , , , 2, 2"
call t "a.0 = 4; a.1 = 'zb'; a.2 = 'ya'; a.3 = 'x'; a.4 = 'yab'", "call SysStemSort 'a.', , , , , 5"
call t "a.0 = 4; a.1 = 'abc'; a.2 = 'ab'; a.3 = 'abd'; a.4 = 'a'", "call SysStemSort 'a.', , , , , 1, 2"
call t "a.0 = 4; a.1 = 'abc'; a.2 = 'ab'; a.3 = 'abd'; a.4 = 'a'", "call SysStemSort 'a.', 'D', 'I', , , 2, 3"
call t "a.0 = 3; a.1 = 'b'; a.2 = 'é'; a.3 = 'A'", "call SysStemSort 'a.'"
call t "a.0 = 3; a.1 = 'b'; a.2 = 'é'; a.3 = 'A'", "call SysStemSort 'a.', , 'I'"
call t "a.0 = 3; a.1 = 'b'; a.2 = '_'; a.3 = 'A'", "call SysStemSort 'a.', , 'I'"
call t "a.0 = 3; a.1 = 'b'; a.2 = '_'; a.3 = 'A'", "call SysStemSort 'a.', , 'I', , , 1, 1"
call t "a.0 = 3; a.1 = 'a'||'00'x; a.2 = 'a'; a.3 = 'a'||'01'x", "call SysStemSort 'a.', , , , , 1, 3"
call t "a.0 = 2; a.1 = 'a'||'00'x; a.2 = 'a'", "call SysStemSort 'a.', , , , , 1, 2"
call t "a.0 = 2; a.1 = 'a'; a.2 = 'aé'", "call SysStemSort 'a.', , 'I', , , 1, 5"
call t "a.0 = 2; a.1 = 'a'; a.2 = 'aé'", "call SysStemSort 'a.', , 'I'"
call t "a.0 = 12; do i = 1 to 12; a.i = (i * 7) // 12; end", "call SysStemSort 'a.'"
call t "a.0 = 10; do i = 1 to 10; a.i = word('k j i h g f e d c b', i); end", "call SysStemSort 'a.', 'D'"
call t "a.0 = 9; do i = 1 to 9; a.i = word('B a A b C c a B A', i) || i; end", "call SysStemSort 'a.', , 'I', , , 1, 1"
call t "a.0 = 0", "call SysStemSort 'a.'"
call t "a.0 = 1; a.1 = 'only'", "call SysStemSort 'a.'"
call t "f = 1; a.foo.0 = 3; a.foo.1 = 'b'; a.foo.2 = 'c'; a.foo.3 = 'a'", "call SysStemSort 'a.foo'"
call t "f = 1; a.foo.0 = 3; a.foo.1 = 'b'; a.foo.2 = 'c'; a.foo.3 = 'a'", "call SysStemSort 'a.foo.'"
call t "f = 1; a.FOO.0 = 2; a.FOO.1 = 'b'; a.FOO.2 = 'a'; a.foo.1 = 'x'", "call SysStemSort 'a.Foo'"
call t "a. = 'd'; a.0 = 2; a.1 = 'q'; a.2 = 'p'", "call SysStemSort 'a.'"
call t "a.0 = '2.0'; a.1 = 'q'; a.2 = 'p'", "call SysStemSort 'a.'"
call t "a.0 = ' 2 '; a.1 = 'q'; a.2 = 'p'", "call SysStemSort 'a.'"
call t "a.0 = 2; a.1 = 'q'; a.2 = 'p'; a.x = 'r'; a.1.5 = 's'; a.3 = 'o'", "call SysStemSort 'a.'"
call t "a.0 = 2; a.1 = .array~of('z', 'y'); a.2 = 'p'", "call SysStemSort 'a.'; say a.1~class~id a.2~class~id"
call t "a.0 = 3; a.1 = 10; a.2 = 9; a.3 = 100", "call SysStemSort 'a.'"
call t "a.0 = 2; a.1 = 'q'; a.2 = 'p'", "r = SysStemSort('a.'); say 'returned' r"
-- the routine's own argument errors
call t "a.0 = 2; a.1 = 'q'; a.2 = 'p'", "call SysStemSort '*'"
call t "a.0 = 2; a.1 = 'q'; a.2 = 'p'", "call SysStemSort '1.x'"
call t "a.0 = 2; a.1 = 'q'; a.2 = 'p'", "call SysStemSort 'a.', 'Q'"
call t "a.0 = 2; a.1 = 'q'; a.2 = 'p'", "call SysStemSort 'a.', ''"
call t "a.0 = 2; a.1 = 'q'; a.2 = 'p'", "call SysStemSort 'a.', 'D', 'N'"
call t "a.0 = 2; a.1 = 'q'; a.2 = 'p'", "call SysStemSort 'a.', 'Q', 'N'"
call t "a.0 = 2; a.1 = 'q'; a.2 = 'p'", "call SysStemSort 'a.', , , 2, 1"
call t "a.0 = 2; a.1 = 'q'; a.2 = 'p'", "call SysStemSort 'a.', , , , , 3, 2"
call t "a.0 = 2; a.1 = 'q'; a.2 = 'p'", "call SysStemSort 'a.', , , 0"
call t "a.0 = 2; a.1 = 'q'; a.2 = 'p'", "call SysStemSort 'a.', , , , 'x'"
call t "a.0 = 2; a.1 = 'q'; a.2 = 'p'", "call SysStemSort 'a.', , , , 1000000000000000000"
call t "a.0 = 2; a.1 = 'q'; a.2 = 'p'", "call SysStemSort 'a.', , , , , , -1"
call t "a.0 = 2; a.1 = 'q'; a.2 = 'p'", "call SysStemSort 'a.', 'A', 'C', 1, 2, 1, 2, 'x'"
call t "a.0 = 2; a.1 = 'q'; a.2 = 'p'", "call SysStemSort"
-- raised from inside the sort: the count, the range and a missing element
call t "a. = 'd'", "call SysStemSort 'a.'"
call t "a.1 = 'q'", "call SysStemSort 'a.'"
call t "a.0 = 'x'; a.1 = 'q'", "call SysStemSort 'a.'"
call t "a.0 = ''; a.1 = 'q'", "call SysStemSort 'a.'"
call t "a.0 = -1", "call SysStemSort 'a.'"
call t "a.0 = '2.5'; a.1 = 'q'; a.2 = 'p'", "call SysStemSort 'a.'"
call t "a.0 = 1234567890; a.1 = 'q'", "call SysStemSort 'a.'"
call t "a.0 = 2; a.1 = 'q'; a.2 = 'p'", "call SysStemSort 'a.', , , 3"
call t "a.0 = 2; a.1 = 'q'; a.2 = 'p'", "call SysStemSort 'a.', , , 1, 3"
call t "a.0 = 3; a.1 = 'q'; a.3 = 'p'", "call SysStemSort 'a.'"
call t "a.0 = '1E1'; a.1 = 'q'; a.2 = 'p'", "call SysStemSort 'a.'"
call t "a. = 'd'; a.0 = 2; a.1 = 'q'", "call SysStemSort 'a.'"
call t "a.0 = 2; a.01 = 'q'; a.02 = 'p'", "call SysStemSort 'a.'"
call t "a.0 = 2; a.1 = 'q'; a.2 = 'p'; drop a.2", "call SysStemSort 'a.'"
-- an error the routine raises before it sorts, untrapped
a.0 = 1; a.1 = 'q'
call SysStemSort 'a.', 'X'
exit

t: procedure
  parse arg setup, code
  f = 0
  interpret setup
  signal on syntax name bad
  interpret code
  say 'ok   ' code '|' show()
  return
bad:
  c = condition('O')
  say 'error' code '|' c~code c~message '|' show()
  return

show: procedure expose a. f
  if datatype(a.0, 'W') & a.0 >= 0 & a.0 < 20 then n = a.0
  else n = 3
  o = a.0 ':'
  do i = 1 to n; o = o a.i; end
  if f then do i = 1 to a.FOO.0; o = o a.FOO.i; end
  return o
