/* extracted from DateFormatter::TestDayElements */
::routine main public

  self~assertSame("1", .DateFormatter~format(.DateTime~fromStandardDate(20190101), "d"))
  self~assertSame("11", .DateFormatter~format(.DateTime~fromStandardDate(20190111), "d"))
  self~assertSame("01", .DateFormatter~format(.DateTime~fromStandardDate(20190101), "dd"))
  self~assertSame("11", .DateFormatter~format(.DateTime~fromStandardDate(20190111), "dd"))

  self~assertSame("1", .DateFormatter~format(.DateTime~fromStandardDate(20190101), "ddd"))
  self~assertSame("001", .DateFormatter~format(.DateTime~fromStandardDate(20190101), "dddd"))

  self~assertSame("365", .DateFormatter~format(.DateTime~fromStandardDate(20191231), "ddd"))
  self~assertSame("365", .DateFormatter~format(.DateTime~fromStandardDate(20191231), "dddd"))

  self~assertSame("366", .DateFormatter~format(.DateTime~fromStandardDate(20201231), "ddd"))
  self~assertSame("366", .DateFormatter~format(.DateTime~fromStandardDate(20201231), "dddd"))

  self~assertSame("1st", .DateFormatter~format(.DateTime~fromStandardDate(20190101), "ddddd"))
  self~assertSame("2nd", .DateFormatter~format(.DateTime~fromStandardDate(20190102), "ddddd"))
  self~assertSame("3rd", .DateFormatter~format(.DateTime~fromStandardDate(20190103), "ddddd"))
  self~assertSame("4th", .DateFormatter~format(.DateTime~fromStandardDate(20190104), "ddddd"))
  self~assertSame("5th", .DateFormatter~format(.DateTime~fromStandardDate(20190105), "ddddd"))
  self~assertSame("6th", .DateFormatter~format(.DateTime~fromStandardDate(20190106), "ddddd"))
  self~assertSame("7th", .DateFormatter~format(.DateTime~fromStandardDate(20190107), "ddddd"))
  self~assertSame("8th", .DateFormatter~format(.DateTime~fromStandardDate(20190108), "ddddd"))
  self~assertSame("9th", .DateFormatter~format(.DateTime~fromStandardDate(20190109), "ddddd"))
  self~assertSame("10th", .DateFormatter~format(.DateTime~fromStandardDate(20190110), "ddddd"))
  self~assertSame("11th", .DateFormatter~format(.DateTime~fromStandardDate(20190111), "ddddd"))
  self~assertSame("12th", .DateFormatter~format(.DateTime~fromStandardDate(20190112), "ddddd"))
  self~assertSame("13th", .DateFormatter~format(.DateTime~fromStandardDate(20190113), "ddddd"))
  self~assertSame("14th", .DateFormatter~format(.DateTime~fromStandardDate(20190114), "ddddd"))
  self~assertSame("15th", .DateFormatter~format(.DateTime~fromStandardDate(20190115), "ddddd"))
  self~assertSame("16th", .DateFormatter~format(.DateTime~fromStandardDate(20190116), "ddddd"))
  self~assertSame("17th", .DateFormatter~format(.DateTime~fromStandardDate(20190117), "ddddd"))
  self~assertSame("18th", .DateFormatter~format(.DateTime~fromStandardDate(20190118), "ddddd"))
  self~assertSame("19th", .DateFormatter~format(.DateTime~fromStandardDate(20190119), "ddddd"))
  self~assertSame("20th", .DateFormatter~format(.DateTime~fromStandardDate(20190120), "ddddd"))
  self~assertSame("21st", .DateFormatter~format(.DateTime~fromStandardDate(20190121), "ddddd"))
  self~assertSame("22nd", .DateFormatter~format(.DateTime~fromStandardDate(20190122), "ddddd"))
  self~assertSame("23rd", .DateFormatter~format(.DateTime~fromStandardDate(20190123), "ddddd"))
  self~assertSame("24th", .DateFormatter~format(.DateTime~fromStandardDate(20190124), "ddddd"))
  self~assertSame("25th", .DateFormatter~format(.DateTime~fromStandardDate(20190125), "ddddd"))
  self~assertSame("26th", .DateFormatter~format(.DateTime~fromStandardDate(20190126), "ddddd"))
  self~assertSame("27th", .DateFormatter~format(.DateTime~fromStandardDate(20190127), "ddddd"))
  self~assertSame("28th", .DateFormatter~format(.DateTime~fromStandardDate(20190128), "ddddd"))
  self~assertSame("29th", .DateFormatter~format(.DateTime~fromStandardDate(20190129), "ddddd"))
  self~assertSame("30th", .DateFormatter~format(.DateTime~fromStandardDate(20190130), "ddddd"))
  self~assertSame("31st", .DateFormatter~format(.DateTime~fromStandardDate(20190131), "ddddd"))


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
