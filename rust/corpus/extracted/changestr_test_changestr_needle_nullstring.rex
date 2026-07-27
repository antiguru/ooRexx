/* extracted from changestr::test_changestr_needle_nullstring */
::routine main public
  string = "string"
  m = .MutableBuffer~new(string)
  self~assertSame(string, m~changeStr("", ""))
  self~assertSame(string, m~changeStr("", "", 0))
  self~assertSame(string, m~changeStr("", "", 1))
  self~assertSame(string, m~changeStr("", "newneedle"))
  self~assertSame(string, m~changeStr("", "newneedle", 0))
  self~assertSame(string, m~changeStr("", "newneedle", 1))

-- changeStr() has three code paths which we test here:
--   needle and newneedle are of the same length
--   newneedle is shorter than needle: result string will get shorter
--   newneedle is longer than needle: result string will get longer
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
