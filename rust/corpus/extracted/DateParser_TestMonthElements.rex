/* extracted from DateParser::TestMonthElements */
::routine main public

  self~assertSame(.DateTime~fromStandardDate(20190731), .DateParser~parse("2019/07/31", "yyyy/MM/dd"))
  self~assertSame(.DateTime~fromStandardDate(20190731), .DateParser~parse("2019/7/31", "yyyy/M/dd"))
  self~assertSame(.DateTime~fromStandardDate(20190731), .DateParser~parse("2019/07/31", "yyyy/M/dd"))

  self~assertSame(.DateTime~fromStandardDate(20190111), .DateParser~parse("Jan 11, 2019", "MMM dd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190211), .DateParser~parse("Feb 11, 2019", "MMM dd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190311), .DateParser~parse("Mar 11, 2019", "MMM dd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190411), .DateParser~parse("Apr 11, 2019", "MMM dd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190511), .DateParser~parse("May 11, 2019", "MMM dd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190611), .DateParser~parse("Jun 11, 2019", "MMM dd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190711), .DateParser~parse("Jul 11, 2019", "MMM dd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190811), .DateParser~parse("Aug 11, 2019", "MMM dd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190911), .DateParser~parse("Sep 11, 2019", "MMM dd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20191011), .DateParser~parse("Oct 11, 2019", "MMM dd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20191111), .DateParser~parse("Nov 11, 2019", "MMM dd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20191211), .DateParser~parse("Dec 11, 2019", "MMM dd, yyyy"))

  self~assertSame(.DateTime~fromStandardDate(20190111), .DateParser~parse("January 11, 2019", "MMMM dd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190211), .DateParser~parse("February 11, 2019", "MMMM dd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190311), .DateParser~parse("March 11, 2019", "MMMM dd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190411), .DateParser~parse("April 11, 2019", "MMMM dd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190511), .DateParser~parse("May 11, 2019", "MMMM dd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190611), .DateParser~parse("June 11, 2019", "MMMM dd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190711), .DateParser~parse("July 11, 2019", "MMMM dd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190811), .DateParser~parse("August 11, 2019", "MMMM dd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20190911), .DateParser~parse("September 11, 2019", "MMMM dd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20191011), .DateParser~parse("October 11, 2019", "MMMM dd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20191111), .DateParser~parse("November 11, 2019", "MMMM dd, yyyy"))
  self~assertSame(.DateTime~fromStandardDate(20191211), .DateParser~parse("December 11, 2019", "MMMM dd, yyyy"))

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
