/* extracted from METHOD::TestRaiseException01 */
::routine main public
  tester = .METHODTester~new
  c = tester~TestRaiseExceptionX(3000, .array~new(0))
  self~assertSame(3, c~rc)
  self~assertSame(3.0, c~code)
  self~assertSame(0, c~additional~size)
  -- native code must have continued after Raise.. call
  self~assertSame(.true, tester~TestGetObjectVariableOrNil("CONTINUE"))

  c = tester~TestRaiseExceptionX(99925, .array~of("Fred"))
  self~assertSame(99, c~rc)
  self~assertSame(99.925, c~code)
  self~assertSame(1, c~additional~size)
  self~assertSame("Fred", c~additional[1])

  c = tester~TestRaiseExceptionX(13001, .array~of("F", c2x("F")))
  self~assertSame(13, c~rc)
  self~assertSame(13.1, c~code)
  self~assertSame(2, c~additional~size)
  self~assertSame("F", c~additional[1])
  self~assertSame(c2x("F"), c~additional[2])

  c = tester~TestRaiseExceptionX(40901, .array~of("FRED", 80, "yada"))
  self~assertSame(40, c~rc)
  self~assertSame(40.901, c~code)
  self~assertSame(3, c~additional~size)
  self~assertSame("FRED", c~additional[1])
  self~assertSame(80, c~additional[2])
  self~assertSame("yada", c~additional[3])

  c = tester~TestRaiseExceptionX(88907, .array~of("min", 80, 100, 0))
  self~assertSame(88, c~rc)
  self~assertSame(88.907, c~code)
  self~assertSame(4, c~additional~size)
  self~assertSame("min", c~additional[1])
  self~assertSame(80, c~additional[2])
  self~assertSame(100, c~additional[3])
  self~assertSame(0, c~additional[4])

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
