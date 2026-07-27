/* extracted from DateParser::TestDayElements */
::routine main public

  self~assertSame(.DateTime~fromStandardDate(20190101), .DateParser~parse("Jan 1, 2019", "MMM d, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190101), .DateParser~parse("Jan 01, 2019", "MMM d, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190101), .DateParser~parse("Jan 01, 2019", "MMM dd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190111), .DateParser~parse("Jan 11, 2019", "MMM d, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190111), .DateParser~parse("Jan 11, 2019", "MMM dd, yyyy"))

  self~assertSame(.DateTime~fromStandardDate(20190101), .DateParser~parse("Jan 1st, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190102), .DateParser~parse("Jan 2nd, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190103), .DateParser~parse("Jan 3rd, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190104), .DateParser~parse("Jan 4th, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190105), .DateParser~parse("Jan 5th, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190106), .DateParser~parse("Jan 6th, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190107), .DateParser~parse("Jan 7th, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190108), .DateParser~parse("Jan 8th, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190109), .DateParser~parse("Jan 9th, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190110), .DateParser~parse("Jan 10th, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190111), .DateParser~parse("Jan 11th, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190112), .DateParser~parse("Jan 12th, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190113), .DateParser~parse("Jan 13th, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190114), .DateParser~parse("Jan 14th, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190115), .DateParser~parse("Jan 15th, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190116), .DateParser~parse("Jan 16th, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190117), .DateParser~parse("Jan 17th, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190118), .DateParser~parse("Jan 18th, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190119), .DateParser~parse("Jan 19th, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190120), .DateParser~parse("Jan 20th, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190121), .DateParser~parse("Jan 21st, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190122), .DateParser~parse("Jan 22nd, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190123), .DateParser~parse("Jan 23rd, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190124), .DateParser~parse("Jan 24th, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190125), .DateParser~parse("Jan 25th, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190126), .DateParser~parse("Jan 26th, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190127), .DateParser~parse("Jan 27th, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190128), .DateParser~parse("Jan 28th, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190129), .DateParser~parse("Jan 29th, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190130), .DateParser~parse("Jan 30th, 2019", "MMM ddddd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190131), .DateParser~parse("Jan 31st, 2019", "MMM ddddd, yyyy"))

  self~assertSame(.DateTime~fromStandardDate(20190101), .DateParser~parse("2019-1", "yyyy-ddd"))
  self~assertSame(.DateTime~fromStandardDate(20190101), .DateParser~parse("2019-001", "yyyy-ddd"))
  self~assertSame(.DateTime~fromStandardDate(20190101), .DateParser~parse("2019-001", "yyyy-dddd"))

  self~assertSame(.DateTime~fromStandardDate(20191231), .DateParser~parse("2019-365", "yyyy-ddd"))
  self~assertSame(.DateTime~fromStandardDate(20191231), .DateParser~parse("2019-365", "yyyy-dddd"))

  self~assertSame(.DateTime~fromStandardDate(20201231), .DateParser~parse("2020-366", "yyyy-ddd"))
  self~assertSame(.DateTime~fromStandardDate(20201231), .DateParser~parse("2020-366", "yyyy-dddd"))


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
