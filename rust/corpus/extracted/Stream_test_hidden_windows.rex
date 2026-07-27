/* extracted from Stream::test_hidden_windows */
::routine main public
  if \.RexxInfo~platform~caselessStartsWith("windows") then
    return

  hidden = .TemporaryTestFile~new(, "test_hidden")~create("hidden")
  "attrib +h" hidden~absolutePath
  self~assertTrue(hidden~isHidden)
  s = .Stream~new(hidden)
  do 2 -- once for hidden, and again for system
    self~assertSame(hidden~absolutePath, s~query("exists"))
    s~open("read shared")
    self~assertSame(hidden~absolutePath, s~query("exists"))
    self~assertSame("hidden", s~charin(, 10))
    s~close
    "attrib -h +s" hidden~absolutePath
  end
  hidden~delete

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
