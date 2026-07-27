/* extracted from METHOD::TestGetAllStemElements01 */
::routine main public
  tester = .METHODTester~new
  d = tester~TestGetAllStemElements(a.)
  self~assertSame(0, d~items)
  a.1 = 1
  a.["abc"] = 2
  a.abc = 3

  d = tester~TestGetAllStemElements(a.)
  self~assertSame(3, d~items)
  self~assertSame(1, d[1])
  self~assertSame(2, d["abc"])
  self~assertSame(3, d["ABC"])

  drop a.abc
  d = tester~TestGetAllStemElements(a.)
  self~assertSame(2, d~items)
  self~assertSame(1, d[1])
  self~assertSame(2, d["abc"])
  self~assertSame(.nil, d["ABC"])

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
