/* extracted from DateFormatter::TestMonthElements */
::routine main public

  self~assertSame("07", .DateFormatter~format(.DateTime~fromStandardDate(20190731), "MM"))
  self~assertSame("7", .DateFormatter~format(.DateTime~fromStandardDate(20190731), "M"))
  self~assertSame("11", .DateFormatter~format(.DateTime~fromStandardDate(20191130), "MM"))
  self~assertSame("11", .DateFormatter~format(.DateTime~fromStandardDate(20191130), "M"))

  self~assertSame("Jan", .DateFormatter~format(.DateTime~fromStandardDate(20190111), "MMM"))
  self~assertSame("Feb", .DateFormatter~format(.DateTime~fromStandardDate(20190211), "MMM"))
  self~assertSame("Mar", .DateFormatter~format(.DateTime~fromStandardDate(20190311), "MMM"))
  self~assertSame("Apr", .DateFormatter~format(.DateTime~fromStandardDate(20190411), "MMM"))
  self~assertSame("May", .DateFormatter~format(.DateTime~fromStandardDate(20190511), "MMM"))
  self~assertSame("Jun", .DateFormatter~format(.DateTime~fromStandardDate(20190611), "MMM"))
  self~assertSame("Jul", .DateFormatter~format(.DateTime~fromStandardDate(20190711), "MMM"))
  self~assertSame("Aug", .DateFormatter~format(.DateTime~fromStandardDate(20190811), "MMM"))
  self~assertSame("Sep", .DateFormatter~format(.DateTime~fromStandardDate(20190911), "MMM"))
  self~assertSame("Oct", .DateFormatter~format(.DateTime~fromStandardDate(20191011), "MMM"))
  self~assertSame("Nov", .DateFormatter~format(.DateTime~fromStandardDate(20191111), "MMM"))
  self~assertSame("Dec", .DateFormatter~format(.DateTime~fromStandardDate(20191211), "MMM"))

  self~assertSame("January",   .DateFormatter~format(.DateTime~fromStandardDate(20190111), "MMMM"))
  self~assertSame("February",  .DateFormatter~format(.DateTime~fromStandardDate(20190211), "MMMM"))
  self~assertSame("March",     .DateFormatter~format(.DateTime~fromStandardDate(20190311), "MMMM"))
  self~assertSame("April",     .DateFormatter~format(.DateTime~fromStandardDate(20190411), "MMMM"))
  self~assertSame("May",       .DateFormatter~format(.DateTime~fromStandardDate(20190511), "MMMM"))
  self~assertSame("June",      .DateFormatter~format(.DateTime~fromStandardDate(20190611), "MMMM"))
  self~assertSame("July",      .DateFormatter~format(.DateTime~fromStandardDate(20190711), "MMMM"))
  self~assertSame("August",    .DateFormatter~format(.DateTime~fromStandardDate(20190811), "MMMM"))
  self~assertSame("September", .DateFormatter~format(.DateTime~fromStandardDate(20190911), "MMMM"))
  self~assertSame("October",   .DateFormatter~format(.DateTime~fromStandardDate(20191011), "MMMM"))
  self~assertSame("November",  .DateFormatter~format(.DateTime~fromStandardDate(20191111), "MMMM"))
  self~assertSame("December",  .DateFormatter~format(.DateTime~fromStandardDate(20191211), "MMMM"))

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
