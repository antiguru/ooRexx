/* extracted from DateParser::TestFractionElements */
::routine main public

  self~assertSame(.DateTime~fromIsoDate("2019-08-02T00:00:00.100000"), .DateParser~parse("2019/08/02 1", "yyyy/MM/dd f"))
  self~assertSame(.DateTime~fromIsoDate("2019-08-02T00:00:00.230000"), .DateParser~parse("2019/08/02 23", "yyyy/MM/dd ff"))
  self~assertSame(.DateTime~fromIsoDate("2019-08-02T00:00:00.345000"), .DateParser~parse("2019/08/02 345", "yyyy/MM/dd fff"))
  self~assertSame(.DateTime~fromIsoDate("2019-08-02T00:00:00.456700"), .DateParser~parse("2019/08/02 4567", "yyyy/MM/dd ffff"))
  self~assertSame(.DateTime~fromIsoDate("2019-08-02T00:00:00.567890"), .DateParser~parse("2019/08/02 56789", "yyyy/MM/dd fffff"))
  self~assertSame(.DateTime~fromIsoDate("2019-08-02T00:00:00.678901"), .DateParser~parse("2019/08/02 678901", "yyyy/MM/dd ffffff"))

  self~assertSame(.DateTime~fromIsoDate("2019-08-02T00:00:00.100000"), .DateParser~parse("2019/08/02 1", "yyyy/MM/dd f"))
  self~assertSame(.DateTime~fromIsoDate("2019-08-02T00:00:00.010000"), .DateParser~parse("2019/08/02 01", "yyyy/MM/dd ff"))
  self~assertSame(.DateTime~fromIsoDate("2019-08-02T00:00:00.001000"), .DateParser~parse("2019/08/02 001", "yyyy/MM/dd fff"))
  self~assertSame(.DateTime~fromIsoDate("2019-08-02T00:00:00.000100"), .DateParser~parse("2019/08/02 0001", "yyyy/MM/dd ffff"))
  self~assertSame(.DateTime~fromIsoDate("2019-08-02T00:00:00.000010"), .DateParser~parse("2019/08/02 00001", "yyyy/MM/dd fffff"))
  self~assertSame(.DateTime~fromIsoDate("2019-08-02T00:00:00.000001"), .DateParser~parse("2019/08/02 000001", "yyyy/MM/dd ffffff"))


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
