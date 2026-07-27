/* extracted from Comparator::TestCaselessDescendingComparator */
::routine main public
  c = .CaselessDescendingComparator~new
  self~assertSame(0, c~compare("abc", "abc"))
  self~assertSame(-1, c~compare("dbc", "abc"))
  self~assertSame(1, c~compare("abc", "dbc"))
  self~assertSame(1, c~compare("ab", "abc"))
  self~assertSame(-1, c~compare("abc", "ab"))

  self~assertSame(0, c~compare("ABC", "abc"))
  self~assertSame(-1, c~compare("DBC", "abc"))
  self~assertSame(1, c~compare("ABC", "dbc"))
  self~assertSame(1, c~compare("AB", "abc"))
  self~assertSame(-1, c~compare("ABC", "ab"))

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
