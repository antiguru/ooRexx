/* extracted from FUNCTION::TestDrop01 */
::routine main public
  expose setVar1 setStem.
  use arg testgroup

  setVar1 = "Mike"
  x = TestDropContextVariable("setVar1")
  self~assertFalse(var("SETVAR1"), "Simple var 1")

  setVar1 = "Mike"
  x = TestDropContextVariable("SETVAR1")
  self~assertFalse(var("SETVAR1"), "Simple var 2")

  setStem.1 = "Mike"
  x = TestDropContextVariable("setStem.")
  signal off novalue
  self~assertSame("SETSTEM.1", setStem.1, "Stem var 1")
  setStem.1 = "Mike"
  x = TestDropContextVariable("SETSTEM.")
  self~assertSame("SETSTEM.1", setStem.1, "Stem var 1")

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
