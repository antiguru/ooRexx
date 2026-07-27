/* extracted from DateFormatter::TestFractionElements */
::routine main public

  self~assertSame("1"     , .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T00:00:00.100000"), "f"))
  self~assertSame("23"    , .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T00:00:00.230000"), "ff"))
  self~assertSame("345"   , .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T00:00:00.345000"), "fff"))
  self~assertSame("4567"  , .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T00:00:00.456700"), "ffff"))
  self~assertSame("56789" , .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T00:00:00.567890"), "fffff"))
  self~assertSame("678901", .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T00:00:00.678901"), "ffffff"))

  self~assertSame("1"     , .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T00:00:00.100000"), "f"))
  self~assertSame("01"    , .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T00:00:00.010000"), "ff"))
  self~assertSame("001"   , .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T00:00:00.001000"), "fff"))
  self~assertSame("0001"  , .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T00:00:00.000100"), "ffff"))
  self~assertSame("00001" , .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T00:00:00.000010"), "fffff"))
  self~assertSame("000001", .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T00:00:00.000001"), "ffffff"))


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
