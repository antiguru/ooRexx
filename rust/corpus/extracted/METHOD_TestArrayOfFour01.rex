/* extracted from METHOD::TestArrayOfFour01 */
::routine main public
  tester = .METHODTester~new
  a = tester~TestArrayOfFour("abc", "def", "ghi", "jkl")
  self~assertSame(4, a~size)
  self~assertSame(4, a~items)
  self~assertSame("abc", a[1])
  self~assertSame("def", a[2])
  self~assertSame("ghi", a[3])
  self~assertSame("jkl", a[4])
  a = tester~TestArrayOfFourAlt("abc", "def", "ghi", "jkl")
  self~assertSame(4, a~size)
  self~assertSame(4, a~items)
  self~assertSame("abc", a[1])
  self~assertSame("def", a[2])
  self~assertSame("ghi", a[3])
  self~assertSame("jkl", a[4])

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
