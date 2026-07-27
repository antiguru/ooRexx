/* extracted from endsWith::test_caselessendsWith */
::routine main public

  self~assertTrue("abc"~caselessendsWith("c"))
  self~assertTrue("abc"~caselessendsWith("C"))
  self~assertTrue("ABC"~caselessendsWith("C"))
  self~assertTrue("abc"~caseLessendsWith("bc"))
  self~assertTrue("abc"~caseLessendsWith("BC"))
  self~assertTrue("abc"~caseLessendsWith("bC"))
  self~assertTrue("abc"~caseLessendsWith("Bc"))
  self~assertTrue("abc"~caselessendsWith("abc"))
  self~assertTrue("abc"~caselessendsWith("ABC"))
  self~assertFalse("abc"~caselessendsWith("dabc"))
  self~assertFalse("abc"~caselessendsWith("abcd"))
  self~assertFalse("abc"~caselessendsWith(""))
  self~assertFalse(""~caselessendsWith("a"))
  self~assertFalse(""~caselessendsWith(""))

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
