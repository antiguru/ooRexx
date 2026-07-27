/* extracted from DateParser::TestOffset */
::routine main public
  self~assertSame(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000+04:00"), .DateParser~parse("2019-08-02T00:00:59.000000+0400", "yyyy-MM-dd'T'hh:mm:ss.ffffffzzz"))
  self~assertSame(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000-04:00"), .DateParser~parse("2019-08-02T00:00:59.000000-0400", "yyyy-MM-dd'T'hh:mm:ss.ffffffzzz"))
  self~assertSame(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000+04:30"), .DateParser~parse("2019-08-02T00:00:59.000000+0430", "yyyy-MM-dd'T'hh:mm:ss.ffffffzzz"))
  self~assertSame(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000+04:30"), .DateParser~parse("2019-08-02T00:00:59.000000+0430", "yyyy-MM-dd'T'hh:mm:ss.ffffffzzz"))
  self~assertSame(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000+04:00"), .DateParser~parse("2019-08-02T00:00:59.000000+04:00", "yyyy-MM-dd'T'hh:mm:ss.ffffffzzzz"))
  self~assertSame(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000-04:00"), .DateParser~parse("2019-08-02T00:00:59.000000-04:00", "yyyy-MM-dd'T'hh:mm:ss.ffffffzzzz"))
  self~assertSame(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000+04:30"), .DateParser~parse("2019-08-02T00:00:59.000000+04:30", "yyyy-MM-dd'T'hh:mm:ss.ffffffzzzz"))
  self~assertSame(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000+04:30"), .DateParser~parse("2019-08-02T00:00:59.000000+04:30", "yyyy-MM-dd'T'hh:mm:ss.ffffffzzzz"))
  self~assertSame(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000Z"), .DateParser~parse("2019-08-02T00:00:59.000000Z", "yyyy-MM-dd'T'hh:mm:ss.ffffffzzz"))
  self~assertSame(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000Z"), .DateParser~parse("2019-08-02T00:00:59.000000Z", "yyyy-MM-dd'T'hh:mm:ss.ffffffzzzz"))

  self~assertSame(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000+04:00"), .DateParser~parse("2019-08-02T00:00:59.000000+04", "yyyy-MM-dd'T'hh:mm:ss.ffffffzz"))
  self~assertSame(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000-04:00"), .DateParser~parse("2019-08-02T00:00:59.000000-04", "yyyy-MM-dd'T'hh:mm:ss.ffffffzz"))
  self~assertSame(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000Z"), .DateParser~parse("2019-08-02T00:00:59.000000Z", "yyyy-MM-dd'T'hh:mm:ss.ffffffzz"))

  self~assertSame(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000+04:00"), .DateParser~parse("2019-08-02T00:00:59.000000+04", "yyyy-MM-dd'T'hh:mm:ss.ffffffz"))
  self~assertSame(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000-04:00"), .DateParser~parse("2019-08-02T00:00:59.000000-04", "yyyy-MM-dd'T'hh:mm:ss.ffffffz"))
  self~assertSame(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000+04:00"), .DateParser~parse("2019-08-02T00:00:59.000000+4", "yyyy-MM-dd'T'hh:mm:ss.ffffffz"))
  self~assertSame(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000-04:00"), .DateParser~parse("2019-08-02T00:00:59.000000-4", "yyyy-MM-dd'T'hh:mm:ss.ffffffz"))
  self~assertSame(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000Z"), .DateParser~parse("2019-08-02T00:00:59.000000Z", "yyyy-MM-dd'T'hh:mm:ss.ffffffz"))

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
