/* extracted from DateParser::TestYearElements */
::routine main public

  self~assertSame(.DateTime~fromStandardDate(20190731), .DateParser~parse("2019/07/31", "yyy/MM/dd"))
  self~assertSame(.DateTime~fromStandardDate(00190731), .DateParser~parse("19/07/31", "yyy/MM/dd"))
  self~assertSame(.DateTime~fromStandardDate(20190731), .DateParser~parse("2019/07/31", "yyyy/MM/dd"))
  self~assertSame(.DateTime~fromStandardDate(00190731), .DateParser~parse("0019/07/31", "yyyy/MM/dd"))
  self~assertSame(.DateTime~fromStandardDate(20190731), .DateParser~parse("2019/07/31", "yyy/MM/dd"))

  -- two digit year tests, which are a little more complicated because they need to be done relative
  -- to the current year. We'll do all of the parsing using the ordered date format

  today = .datetime~new~date
  startRange = today~addYears(-50)
  endRange = today~addYears(49)

  self~assertSame(today, .DateParser~parse(today~orderedDate, "yy/MM/dd"))
  self~assertSame(startRange, .DateParser~parse(startRange~orderedDate, "yy/MM/dd"))
  self~assertSame(endRange, .DateParser~parse(endRange~orderedDate, "yy/MM/dd"))

  -- these should all produce the same results as above
  self~assertSame(today, .DateParser~parse(today~orderedDate, "y/MM/dd"))
  self~assertSame(startRange, .DateParser~parse(startRange~orderedDate, "y/MM/dd"))
  self~assertSame(endRange, .DateParser~parse(endRange~orderedDate, "y/MM/dd"))

  -- testing the no leading zero is a little tricker because of the sliding window. We
  -- will produce a date object by parsing the same date, then use the parser on the retrieved
  -- date to make sure we get the same result

  date = .DateTime~fromOrderedDate('01/01/01')
  -- this strips the leading zero from the date
  self~assertSame(date, .DateParser~parse(date~orderedDate~strip('L', '0'), "y/MM/dd"))


-- test the different ways of parsing off Month elements
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
