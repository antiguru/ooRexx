/* extracted from WindowsEventLog::test_open02 */
::routine main public
  l = .WindowsEventLog~new

  sysNum = l~getNumber( , "System")
  sysFirst = l~getFirst( , "System")
  sysLast = l~getLast( , "System")

  ret = l~open( , "System")
  self~assertSame(0, ret)

  -- Once a log is opened with open, that log should be used for all successive
  -- methods, until it is closed.  Again, not necessarily proof, but a good
  -- indication.

  lNum = l~getNumber
  lFirst = l~getFirst
  lLast = l~getLast

  self~assertSame(sysNum, lNum)
  self~assertSame(sysFirst, lFirst)
  self~assertSame(sysLast, lLast)

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
