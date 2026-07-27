/* extracted from DateTime::testUtcDate */
::routine main public
    a_dateTime = .dateTime~new(2007,10,8,13,10,5,123456,0)
    d = .DateTime~new(a_dateTime~utcDate, 0)

    expected = '2007-10-08T13:10:05.123456Z'
    self~assertSame(expected, d~utcIsoDate)

    a_dateTime = .dateTime~new(2007,10,8,9,10,5,123456,-240)
    d = .DateTime~new(a_dateTime~utcDate, 0)

    expected = '2007-10-08T13:10:05.123456Z'
    self~assertSame(expected, d~utcIsoDate)

    a_dateTime = .dateTime~new(2007,10,8,17,10,5,123456,240)
    d = .DateTime~new(a_dateTime~utcDate, 0)

    expected = '2007-10-08T13:10:05.123456Z'
    self~assertSame(expected, d~utcIsoDate)

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
