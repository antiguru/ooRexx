/* extracted from FUNCTION::test_AddCommandEnvironment_redirecting */
::routine main public
  -- add a redirecting subcommand environment
  call TestAddCommandEnvironment "rc-redirecting", "redirecting"
  address "rc-redirecting" ""
  -- when the handler is installed as "redirecting" its return code
  -- is a five-digit number with digits 0 or 1 in sequence:
  --   IsRedirectionRequested()
  --   IsInputRedirected()
  --   IsOutputRedirected()
  --   IsErrorRedirected()
  --   AreOutputAndErrorSameTarget()
  self~assertEquals(rc, 00000)

  address "rc-redirecting" "" with input using 123
  self~assertSame(rc, 11000)
  address "rc-redirecting" "" with output stem o.
  self~assertSame(rc, 10100)
  address "rc-redirecting" "" with error stem e.
  self~assertSame(rc, 10010)
  address "rc-redirecting" "" with input using 123 output stem o.
  self~assertSame(rc, 11100)
  address "rc-redirecting" "" with input using 123 error stem e.
  self~assertSame(rc, 11010)
  address "rc-redirecting" "" with output stem o. error stem e.
  self~assertSame(rc, 10110)
  address "rc-redirecting" "" with input using 123 output stem o. error stem e.
  self~assertSame(rc, 11110)
  address "rc-redirecting" "" with output stem s. error stem s.
  self~assertSame(rc, 10111)
  address "rc-redirecting" "" with input using 123 output stem s. error stem s.
  self~assertSame(rc, 11111)

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
