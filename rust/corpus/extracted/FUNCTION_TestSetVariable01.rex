/* extracted from FUNCTION::TestSetVariable01 */
::routine main public

  x = TestSetContextVariable('setVar1', "abc")
  self~assertSame("abc", setVar1)

  x = TestSetContextVariable('setVar1', "def")
  self~assertSame("def", setVar1)

  x = TestSetContextVariable('setStem.', .stem~new("FOOBAR."))
  self~assertSame("FOOBAR.", setStem.[])
  signal off novalue
  self~assertSame("FOOBAR.1", setStem.1)

  x = TestSetContextVariable('setStem.', "Fred")
  self~assertSame("Fred", setStem.[])
  self~assertSame("Fred", setStem.1)

  -- This is different than object variables.  All forms are
  -- supported here.
  Stem1.1 = "Mike"
  x = TestSetContextVariable("STEM1.1", "George");

  self~assertSame("George", Stem1.1)

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
