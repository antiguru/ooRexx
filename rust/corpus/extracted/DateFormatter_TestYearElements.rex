/* extracted from DateFormatter::TestYearElements */
::routine main public

  self~assertSame("2019", .DateFormatter~format(.DateTime~fromStandardDate(20190731), "yyy"))
  self~assertSame("19", .DateFormatter~format(.DateTime~fromStandardDate(00190731), "yyy"))
  self~assertSame("2019", .DateFormatter~format(.DateTime~fromStandardDate(20190731), "yyyy"))
  self~assertSame("0019", .DateFormatter~format(.DateTime~fromStandardDate(00190731), "yyyy"))
  self~assertSame("2019", .DateFormatter~format(.DateTime~fromStandardDate(20190731), "yyyy"))

  self~assertSame("19", .DateFormatter~format(.DateTime~fromStandardDate(20190731), "y"))
  self~assertSame("9", .DateFormatter~format(.DateTime~fromStandardDate(20090731), "y"))
  self~assertSame("9", .DateFormatter~format(.DateTime~fromStandardDate(00090731), "y"))
  self~assertSame("19", .DateFormatter~format(.DateTime~fromStandardDate(20190731), "yy"))
  self~assertSame("09", .DateFormatter~format(.DateTime~fromStandardDate(20090731), "yy"))
  self~assertSame("09", .DateFormatter~format(.DateTime~fromStandardDate(00090731), "yy"))


-- test the different ways of formatting Month elements
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
