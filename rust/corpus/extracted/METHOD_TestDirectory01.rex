/* extracted from METHOD::TestDirectory01 */
::routine main public
  tester = .METHODTester~new
  d = tester~TestNewDirectory
  self~assertTrue(d~isA(.Directory))
  self~assertTrue(tester~TestIsDirectory(d))
  tester~TestDirectoryPut(d, 1, 'abc')
  tester~TestDirectoryPut(d, 2, 'ABC')
  self~assertSame(1, d['abc'], "Test 1")
  self~assertSame(2, d['ABC'], "Test 2")
  self~assertSame(1, tester~TestDirectoryAt(d, 'abc'), "Test 3")
  self~assertSame(2, tester~TestDirectoryAt(d, 'ABC'), "Test 4")
  self~assertSame(1, tester~TestDirectoryRemove(d, 'abc'))
  self~assertSame(.nil, d['abc'], "Test 5")

  self~assertFalse(tester~TestIsDirectory("abc"))
  self~assertFalse(tester~TestIsDirectory(.nil))

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
