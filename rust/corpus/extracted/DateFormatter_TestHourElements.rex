/* extracted from DateFormatter::TestHourElements */
::routine main public
  self~assertSame("1", .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T01:00:00.000000"), "h"))
  self~assertSame("12", .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T00:00:00.000000"), "h"))

  self~assertSame("01", .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T01:00:00.000000"), "H"))
  self~assertSame("12", .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T00:00:00.000000"), "H"))

  self~assertSame("0" , .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T00:00:00.000000"), "hh"))
  self~assertSame("1" , .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T01:00:00.000000"), "hh"))
  self~assertSame("12", .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T12:00:00.000000"), "hh"))
  self~assertSame("13", .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T13:00:00.000000"), "hh"))
  self~assertSame("23", .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T23:00:00.000000"), "hh"))

  self~assertSame("00", .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T00:00:00.000000"), "HH"))
  self~assertSame("01", .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T01:00:00.000000"), "HH"))
  self~assertSame("12", .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T12:00:00.000000"), "HH"))
  self~assertSame("13", .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T13:00:00.000000"), "HH"))
  self~assertSame("23", .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T23:00:00.000000"), "HH"))


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
