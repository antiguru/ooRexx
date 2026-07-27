/* extracted from TimeSpan::test_string */
::routine main public
    self~assertSame("00:00:00.000000", .TimeSpan~new(0))
    self~assertSame("00:00:00.000100", .TimeSpan~new(100))

    self~assertSame("00:00:00.000000", .TimeSpan~new(0, 0, 0))
    self~assertSame("00:00:01.000000", .TimeSpan~new(0, 0, 1))
    self~assertSame("09:00:00.000000", .TimeSpan~new(9, 0, 0))
    self~assertSame("09:02:01.000000", .TimeSpan~new(9, 2, 1))

    self~assertSame("00:00:00.000000", .TimeSpan~new(0, 0, 0, 0))
    self~assertSame("09:00:00.000000", .TimeSpan~new(0, 9, 0, 0))
    self~assertSame("1.00:00:00.000000", .TimeSpan~new(1, 0, 0, 0))
    self~assertSame("1.02:00:00.000000", .TimeSpan~new(1, 2, 0, 0))
    self~assertSame("1.00:04:00.000000", .TimeSpan~new(1, 0, 4, 0))
    self~assertSame("1.00:00:08.000000", .TimeSpan~new(1, 0, 0, 8))
    self~assertSame("1.02:04:08.000000", .TimeSpan~new(1, 2, 4, 8))

    self~assertSame("00:00:00.000000", .TimeSpan~new(0, 0, 0, 0, 0))
    self~assertSame("00:00:00.000123", .TimeSpan~new(0, 0, 0, 0, 123))
    self~assertSame("900.00:00:00.000000", .TimeSpan~new(900, 0, 0, 0, 0))
    self~assertSame("900.00:00:00.000123", .TimeSpan~new(900, 0, 0, 0, 123))
    self~assertSame("900.00:00:59.000000", .TimeSpan~new(900, 0, 0, 59, 0))
    self~assertSame("900.00:59:00.000000", .TimeSpan~new(900, 0, 59, 0, 0))
    self~assertSame("900.23:00:00.000000", .TimeSpan~new(900, 23, 0, 0, 0))
    self~assertSame("900.23:59:59.999999", .TimeSpan~new(900, 23, 59, 59, 999999))

    self~assertSame( "04:33:15.000000",   .TimeSpan~new(4, 33, 15))
    self~assertSame("-04:33:15.000000", - .TimeSpan~new(4, 33, 15))
    self~assertSame( "6.04:33:15.000100",   .TimeSpan~new(6, 4, 33, 15, 100))
    self~assertSame("-6.04:33:15.000100", - .TimeSpan~new(6, 4, 33, 15, 100))


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
