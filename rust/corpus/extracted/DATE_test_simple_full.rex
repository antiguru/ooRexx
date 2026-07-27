/* extracted from DATE::test_simple_full */
::routine main public
  numeric digits 18
  self~assertTrue(date("full")~dataType("Num"), date("full") "should be a number")
  self~assertTrue(date("F") > 63799000000000000 & date("f") < 64344000000000000, "63799000000000000 <" date("F") "< 64344000000000000 expected")
  self~assertSame(0, date("F", "1 Jan 0001"))
  -- if we add the 31 Dec 9999 and add 366 days for leap year 10000,
  -- we arrive at 10000 years of 365.2425 days/year in the Gregorian calendar
  microsPerDay = 24 * 3600 * 1e6
  days367 = (1 + 366) * microsPerDay
  self~assertEquals(365.2425 * 10000 * microsPerDay, days367 + date("f", "31 Dec 9999"))

-- yyyy-mm-dd
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
