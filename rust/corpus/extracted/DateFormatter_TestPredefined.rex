/* extracted from DateFormatter::TestPredefined */
::routine main public
  self~assertSame("3 Aug 2019", .DateFormatter~format(.DateTime~fromNormalDate("3 Aug 2019"), .Dateformats~NormalDate))
  self~assertSame("2019-08-02T00:00:59.000000", .DateFormatter~format(.DateTime~fromIsoDate("2019-08-02T00:00:59.000000"), .Dateformats~ISODate))
  self~assertSame("2019-08-02T00:00:59.000000+04:00", .DateFormatter~format(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000+04:00"), .Dateformats~UTCISODate))
  self~assertSame("08/03/19", .DateFormatter~format(.DateTime~fromEuropeanDate("08/03/19"), .Dateformats~EuropeanDate))
  self~assertSame("03/08/19", .DateFormatter~format(.DateTime~fromUsaDate("03/08/19"), .Dateformats~USADate))
  self~assertSame("08/03/2019", .DateFormatter~format(.DateTime~fromStandardDate("20190803"), .Dateformats~LongUsaDate))
  self~assertSame("03/08/2019", .DateFormatter~format(.DateTime~fromStandardDate("20190803"), .Dateformats~LongEuropeanDate))
  self~assertSame("19/08/03", .DateFormatter~format(.DateTime~fromStandardDate("20190803"), .Dateformats~OrderedDate))
  self~assertSame("2019/08/03", .DateFormatter~format(.DateTime~fromStandardDate("20190803"), .Dateformats~LongOrderedDate))
  self~assertSame("2019/08/03", .DateFormatter~format(.DateTime~fromStandardDate("20190803"), .Dateformats~StandardDate))
  self~assertSame("2019-250", .DateFormatter~format(.DateTime~fromOrdinalDate("2019-250"), .Dateformats~OrdinalDate))
  self~assertSame("2019-W25-1", .DateFormatter~format(.DateTime~fromWeekNumberDate("2019-W25-1"), .Dateformats~WeekNumberDate))


  self~assertSame("1:24am", .DateFormatter~format(.DateTime~fromCivilTime("1:24am"), .Dateformats~CivilTime))
  self~assertSame("1:24pm", .DateFormatter~format(.DateTime~fromCivilTime("1:24pm"), .Dateformats~CivilTime))
  self~assertSame("13:24:00", .DateFormatter~format(.DateTime~fromNormalTime("13:24:00"), .Dateformats~NormalTime))
  self~assertSame("1:24:00", .DateFormatter~format(.DateTime~fromNormalTime("01:24:00"), .Dateformats~NormalTime))
  self~assertSame("13:24:00.123456", .DateFormatter~format(.DateTime~fromLongTime("13:24:00.123456"), .Dateformats~LongTime))
  self~assertSame("01:24:00.123456", .DateFormatter~format(.DateTime~fromLongTime("01:24:00.123456"), .Dateformats~LongTime))

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
