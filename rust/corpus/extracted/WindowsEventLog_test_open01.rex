/* extracted from WindowsEventLog::test_open01 */
::routine main public
  l = .WindowsEventLog~new

  -- With no args, open should open the application log.  So, first get the
  -- numbers for the application log, the do the open, then get the numbers
  -- again and they should match.  This will not prove absolutely that the open
  -- with no args opened the application log, but it will be a good indication.
  -- And, if the test fails, it will show that something is not working.

  appNum = l~getNumber( , "Application")
  appFirst = l~getFirst( , "Application")
  appLast = l~getLast( , "Application")

  ret = l~open
  self~assertSame(0, ret)

  lNum = l~getNumber
  lFirst = l~getFirst
  lLast = l~getLast

  self~assertSame(appNum, lNum)
  self~assertSame(appFirst, lFirst)
  self~assertSame(appLast, lLast)

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
