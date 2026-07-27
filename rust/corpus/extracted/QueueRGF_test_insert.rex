/* extracted from QueueRGF::test_insert */
::routine main public
  expose collDir clz bUserDefinedIndex bSingleItem o1 o2

  a=clz~new
  a~insert("a")      -- insert at end
  self~assertTrue(a~hasItem("a"))
  self~assertTrue(a~hasIndex(1))
  self~assertEquals("a", a[1])

  a~insert("b")      -- insert at end
  self~assertTrue(a~hasItem("b"))
  self~assertTrue(a~hasIndex(2))
  self~assertEquals("a", a[1])
  self~assertEquals("b", a[2])

  a~insert("c",1)    -- insert after first element
  self~assertTrue(a~hasItem("c"))
  self~assertTrue(a~hasIndex(3))
  self~assertEquals("a", a[1])
  self~assertEquals("b", a[3])
  self~assertEquals("c", a[2])

  a~insert("d",.nil) -- insert as first
  self~assertTrue(a~hasItem("c"))
  self~assertTrue(a~hasIndex(3))
  self~assertEquals("a", a[2])
  self~assertEquals("b", a[4])
  self~assertEquals("c", a[3])
  self~assertEquals("d", a[1])



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
