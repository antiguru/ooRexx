/* extracted from subWords::test_many */
::routine main public
  all = "one two three four"~makeArray(" ")
  self~assertEquals(all, "one two three four"~subWords)
  self~assertEquals(all, "one two three four"~subWords(1))
  self~assertEquals(all, "one two three four"~subWords(, 5))
  self~assertEquals(all, (" one  two" || .String~tab || "three" .String~blank "four")~subWords(1, 10))
  self~assertEquals(("one", "two", "three"), "one two three four"~subWords(1, 3))
  self~assertEquals(("one", "two", "three"), "one two three four"~subWords(, 3))
  self~assertEquals(("two", "three", "four"), "one two three four"~subWords(2, 3))
  self~assertEquals(("two", "three", "four"), "one two three four"~subWords(2, 4))



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
