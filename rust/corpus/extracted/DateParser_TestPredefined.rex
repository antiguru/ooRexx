/* extracted from DateParser::TestPredefined */
::routine main public
  self~assertSame(.DateTime~fromNormalDate("3 Aug 2019"), .DateParser~parse("3 Aug 2019", .Dateformats~NormalDate))
  self~assertSame(.DateTime~fromIsoDate("2019-08-02T00:00:59.000000"), .DateParser~parse("2019-08-02T00:00:59.000000", .Dateformats~ISODate))
  self~assertSame(.DateTime~fromUtcIsoDate("2019-08-02T00:00:59.000000+04:00"), .DateParser~parse("2019-08-02T00:00:59.000000+04:00", .Dateformats~UTCISODate))
  self~assertSame(.DateTime~fromEuropeanDate("08/03/19"), .DateParser~parse("08/03/19", .Dateformats~EuropeanDate))
  self~assertSame(.DateTime~fromUsaDate("03/08/19"), .DateParser~parse("03/08/19", .Dateformats~USADate))
  self~assertSame(.DateTime~fromStandardDate("20190803"), .DateParser~parse("08/03/2019", .Dateformats~LongUsaDate))
  self~assertSame(.DateTime~fromStandardDate("20190803"), .DateParser~parse("03/08/2019", .Dateformats~LongEuropeanDate))
  self~assertSame(.DateTime~fromStandardDate("20190803"), .DateParser~parse("19/08/03", .Dateformats~OrderedDate))
  self~assertSame(.DateTime~fromStandardDate("20190803"), .DateParser~parse("2019/08/03", .Dateformats~LongOrderedDate))
  self~assertSame(.DateTime~fromStandardDate("20190803"), .DateParser~parse("2019/08/03", .Dateformats~StandardDate))
  self~assertSame(.DateTime~fromOrdinalDate("2019-250"), .DateParser~parse("2019-250", .Dateformats~OrdinalDate))
  self~assertSame(.DateTime~fromWeekNumberDate("2019-W25-1"), .DateParser~parse("2019-W25-1", .Dateformats~WeekNumberDate))


  self~assertSame(.DateTime~fromCivilTime("1:24am"), .DateParser~parse("1:24am", .Dateformats~CivilTime))
  self~assertSame(.DateTime~fromCivilTime("1:24pm"), .DateParser~parse("1:24pm", .Dateformats~CivilTime))
  self~assertSame(.DateTime~fromNormalTime("13:24:00"), .DateParser~parse("13:24:00", .Dateformats~NormalTime))
  self~assertSame(.DateTime~fromNormalTime("01:24:00"), .DateParser~parse("01:24:00", .Dateformats~NormalTime))
  self~assertSame(.DateTime~fromLongTime("13:24:00.123456"), .DateParser~parse("13:24:00.123456", .Dateformats~LongTime))
  self~assertSame(.DateTime~fromLongTime("01:24:00.123456"), .DateParser~parse("01:24:00.123456", .Dateformats~LongTime))

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
