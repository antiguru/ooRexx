/* extracted from Properties::test_save */
::routine main public
  p = .Properties~new

  p~setLogical("flag", .true)
  lines = .Array~new
  p~save(.ArrayStream~new(lines))
  self~assertSame(1, lines~items)
  self~assertSame("flag=true", lines[1])

  p~empty
  p~load(.ArrayStream~new(lines))
  self~assertSame("true", p~getProperty("flag"))

  p~empty
  p~setProperty("string", " leading & trailing space ")
  lines~empty
  p~save(.ArrayStream~new(lines))
  self~assertSame(1, lines~items)
  self~assertSame("string= leading & trailing space ", lines[1])


-- methods inherited from Directory

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
