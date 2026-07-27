/* extracted from DATE::test_simple_ticks */
::routine main public
  numeric digits 18
  self~assertTrue(date("ticks")~dataType("Num"), date("ticks") "should be a number")
  self~assertTrue(date("T") > 1664000000 & date("T") < 2240000000, "1664000000 <" date("T") "< 2240000000 expected")
  self~assertSame(0, date("t", "1 Jan 1970"))
  -- to find the number of days between 1 Jan 0001 and 1 Jan 1970 we add
  -- 2000 Gregorian years (with 365.2425 days each) to arrive at 1 Jan 2001,
  -- from where we substract 31 years (each with 365 days) and 8 leap days
  -- in this 31-year period
  daysEpoch = 365.2425 * 2000 - 31 * 365 - 8
  self~assertEquals(-daysEpoch * 24 * 3600, date("TICKS", "1 Jan 0001"))
  -- the number of days between 1 Jan 0001 and 31 Dec 9999 is 10000 Gregorian
  -- years (with 365.2425 days each) minus the 366-day year 10000 minus one day.
  -- From that we substract the days to 1 Jan 1970 calculated above.
  days9999 = 365.2425 * 10000 - 366 - 1 - daysEpoch
  self~assertEquals(days9999 * 24 * 3600, date("TICKS", "31 Dec 9999"))

-- mm/dd/yy
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
