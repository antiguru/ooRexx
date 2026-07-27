/* extracted from DateFormatter::TestOffset */
::routine main public
  self~assertSame("+04:00" , .DateFormatter~format(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000+04:00"), "zzzz"))
  self~assertSame("-04:00" , .DateFormatter~format(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000-04:00"), "zzzz"))
  self~assertSame("+04:30" , .DateFormatter~format(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000+04:30"), "zzzz"))
  self~assertSame("+04:30" , .DateFormatter~format(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000+04:30"), "zzzz"))
  self~assertSame("Z", .DateFormatter~format(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000Z"), "zzzz"))

  self~assertSame("+0400" , .DateFormatter~format(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000+04:00"), "zzz"))
  self~assertSame("-0400" , .DateFormatter~format(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000-04:00"), "zzz"))
  self~assertSame("+0430" , .DateFormatter~format(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000+04:30"), "zzz"))
  self~assertSame("+0430" , .DateFormatter~format(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000+04:30"), "zzz"))
  self~assertSame("Z", .DateFormatter~format(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000Z"), "zzz"))

  self~assertSame("+04", .DateFormatter~format(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000+04:00"), "zz"))
  self~assertSame("-04", .DateFormatter~format(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000-04:00"), "zz"))
  self~assertSame("-04", .DateFormatter~format(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000-04:30"), "zz"))
  self~assertSame("+04", .DateFormatter~format(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000+04:30"), "zz"))
  self~assertSame("Z", .DateFormatter~format(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000Z"), "zz"))

  self~assertSame("+4", .DateFormatter~format(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000+04:00"), "z"))
  self~assertSame("+4", .DateFormatter~format(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000+04:30"), "z"))
  self~assertSame("-4", .DateFormatter~format(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000-04:00"), "z"))
  self~assertSame("-4", .DateFormatter~format(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000-04:30"), "z"))
  self~assertSame("Z", .DateFormatter~format(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000Z"), "z"))

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
