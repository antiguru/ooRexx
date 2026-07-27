/* extracted from caselessMatch::test_Match_caseless */
::routine main public

  str = .String~new("abcDEFghi")
  null = .String~new("")
  abc = .String~new("abc")
  self~assertTrue(str~caselessMatch(2, "bc"))
  self~assertTrue(str~caselessMatch(2, "bcD"))
  self~assertTrue(str~caselessMatch(2, "bcd"))
  self~assertFalse(str~caselessMatch(2, "ab"))
  self~assertFalse(str~caselessMatch(2, "cD"))

  self~assertFalse(null~caselessMatch(1, "Abc"))
  self~assertFalse(abc~caselessMatch(4, "abc"))
  self~assertFalse(abc~caselessMatch(1, "abcd"))

  self~assertTrue(str~caselessMatch(2, "dbc", 2))
  self~assertTrue(str~caselessMatch(2, "dbce", 2, 2))
  self~assertFalse(abc~caselessMatch(1, ""))
  self~assertFalse(abc~caselessMatch(1, "abc", 1, 0))

  -- out of bounds should all be false
  self~assertFalse(str~caselessMatch(str~length + 1, "i"))
  self~assertFalse(str~caselessMatch(1, "abc", 4))
  self~assertFalse(str~caselessMatch(1, "abc", 1, 4))

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
