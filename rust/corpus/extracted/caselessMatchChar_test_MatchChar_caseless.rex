/* extracted from caselessMatchChar::test_MatchChar_caseless */
::routine main public

  str = .String~new("abcDEFghi")
  null = .String~new("")
  abc = .String~new("abc")
  self~assertTrue(str~caselessMatchChar(2, "b"))
  self~assertTrue(str~caselessMatchChar(2, "B"))
  self~assertFalse(str~caselessMatchChar(2, "c"))
  self~assertTrue(str~caselessMatchChar(2, "bc"))
  self~assertTrue(str~caselessMatchChar(2, "Bc"))
  self~assertFalse(str~caselessMatchChar(2, "cD"))
  self~assertFalse(null~caselessMatchChar(1, "Abc"))
  self~assertFalse(abc~caselessMatchChar(1, ""))
  self~assertTrue(abc~caselessMatchChar(1, xrange('00'x, 'ff'x)))

  -- out of bounds should all be false
  self~assertFalse(abc~caselessMatchChar(4, "abc"))
  self~assertFalse(str~caselessMatchChar(str~length + 1, ""))

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
