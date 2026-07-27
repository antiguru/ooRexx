/* extracted from DateParser::TestWeekNumber */
::routine main public

  self~assertSame(.DateTime~fromIsoDate("2019-01-01T00:00:00.000000"), .DateParser~parse("2019 1 2", "yyyy w D"))
  self~assertSame(.DateTime~fromIsoDate("2019-01-01T00:00:00.000000"), .DateParser~parse("2019 01 2", "yyyy w D"))
  self~assertSame(.DateTime~fromIsoDate("2019-01-01T00:00:00.000000"), .DateParser~parse("2019 01 2", "yyyy ww D"))
  self~assertSame(.DateTime~fromIsoDate("2019-01-01T00:00:00.000000"), .DateParser~parse("2019 1 Tuesday", "yyyy w DDD"))
  self~assertSame(.DateTime~fromIsoDate("2019-01-01T00:00:00.000000"), .DateParser~parse("2019 01 Tue", "yyyy w DD"))
  self~assertSame(.DateTime~fromIsoDate("2019-01-01T00:00:00.000000"), .DateParser~parse("2019 01 Tuesday", "yyyy ww DDD"))
  self~assertSame(.DateTime~fromIsoDate("2019-01-01T00:00:00.000000"), .DateParser~parse("2019 01 Tue", "yyyy ww DD"))

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
