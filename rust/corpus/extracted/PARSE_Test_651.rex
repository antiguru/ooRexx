/* extracted from PARSE::Test_651 */
::routine main public
   t='1234567890abcdefghijklmnopqrstuvwxyz'
   Parse Var t 0 v1 10
   self~assertSame(v1, '123456789')
   Parse Var t 2 v2 10
   self~assertSame(v2, '23456789')
   Parse Var t +2 v3 10
   self~assertSame(v3, '3456789')
   Parse Var t 10000 -880 v4 10
   self~assertSame(v4, '123456789')
   Parse Var t '' -2 v5 10
   self~assertSame(v5, 'yz')
   Parse Var t '6' v6 +2
   self~assertSame(v6, '67')
   Parse Var t 77 v7 100
   self~assertSame(v7, '')

::class shim public
::method assertEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertNotEquals
  use arg expected, actual
  if expected == actual then do
    say "FAIL not-expected["expected"] actual["actual"]"
    exit 1
  end
::method assertTrue
  use arg condition
  if \condition then do
    say "FAIL expected true actual["condition"]"
    exit 1
  end
::method assertFalse
  use arg condition
  if condition then do
    say "FAIL expected false actual["condition"]"
    exit 1
  end
::method assertNull
  use arg actual
  if actual \== .nil then do
    say "FAIL expected nil actual["actual"]"
    exit 1
  end
::method assertNotNull
  use arg actual
  if actual == .nil then do
    say "FAIL expected non-nil actual nil"
    exit 1
  end
::method assertSame
  use arg expected, actual
  if \(expected == actual) then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertNotSame
  use arg expected, actual
  if expected == actual then do
    say "FAIL not-expected["expected"] actual["actual"]"
    exit 1
  end
::method expectSyntax
  use arg code
  nop
::method assertListEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertArrayEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
