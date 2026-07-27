/* extracted from STREAM::test_close_twice */
::routine main public

  f = "streamCloseTest.delMe"

  -- Lineout should open the file and write the line.
  ret = lineout(f, 'line 1')
  self~assertSame(0, ret)

  -- Close the file, return should be READY: and query state should be unknown
  ret = stream(f, 'C', "CLOSE")
  state = stream(f, "STATE")

  self~assertSame("READY:", ret)
  self~assertSame("UNKNOWN", state)

  -- Close it again. This time the return should be the empty string (docs.)
  ret = stream(f, 'C', "CLOSE")
  state = stream(f, "STATE")

  self~assertSame("", ret)
  self~assertSame("UNKNOWN", state)

  ret = deleteFile(f)

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
