/* extracted from subWords::test_empty */
::routine main public
  empty = .Array~new
  do whitespace over "", " ", .String~tab, .String~tab || " "
    self~assertEquals(empty, whitespace~subWords)
    self~assertEquals(empty, whitespace~subWords(1))
    self~assertEquals(empty, whitespace~subWords(, 0))
    self~assertEquals(empty, whitespace~subWords(, 1))
    self~assertEquals(empty, whitespace~subWords(1, 0))
    self~assertEquals(empty, whitespace~subWords(1, 1))
  end
  self~assertEquals(empty, "_"~subWords(2))
  self~assertEquals(empty, "_"~subWords(, 0))
  self~assertEquals(empty, "_"~subWords(2, 0))
  self~assertEquals(empty, 123~subWords(2, 1))
  self~assertEquals(empty, "one two"~subWords(3))
  self~assertEquals(empty, " 1  2 "~subWords(3))
  self~assertEquals(empty, ("one" .String~tab "two")~subWords(3))
  self~assertEquals(empty, (.String~tab "one" .String~tab "two" .String~tab)~subWords(3))

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
