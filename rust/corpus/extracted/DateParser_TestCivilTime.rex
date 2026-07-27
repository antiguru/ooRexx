/* extracted from DateParser::TestCivilTime */
::routine main public

  self~assertSame(.DateTime~fromIsoDate("2019-08-02T00:30:00.000000"), .DateParser~parse("2019/08/02 12:30am", "yyyy/MM/dd h:mmtt"))
  self~assertSame(.DateTime~fromIsoDate("2019-08-02T12:30:00.000000"), .DateParser~parse("2019/08/02 12:30pm", "yyyy/MM/dd h:mmtt"))

  self~assertSame(.DateTime~fromIsoDate("2019-08-02T00:30:00.000000"), .DateParser~parse("2019/08/02 12:30a", "yyyy/MM/dd h:mmt"))
  self~assertSame(.DateTime~fromIsoDate("2019-08-02T12:30:00.000000"), .DateParser~parse("2019/08/02 12:30p", "yyyy/MM/dd h:mmt"))

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
