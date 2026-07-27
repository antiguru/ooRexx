/* extracted from Relation::test_uniqueindexes */
::routine main public
  r = .relation~new
  i = r~uniqueindexes
  self~assertTrue(i~isa(.array))
  self~assertTrue(i~isEmpty)

  r~put(1, "foo")

  i = r~uniqueindexes
  self~assertEquals(1, i~items)
  self~assertEquals("foo", i[1])

  r~put(2, "bar")

  i = r~uniqueindexes
  self~assertEquals(2, i~items)
  self~assertTrue(i~hasItem("foo"))
  self~assertTrue(i~hasItem("bar"))

  loop i = 1 to 10
      r~put(i, "foo")
      r~put(i, "bar")
  end

  i = r~uniqueindexes
  self~assertEquals(2, i~items)
  self~assertTrue(i~hasItem("foo"))
  self~assertTrue(i~hasItem("bar"))


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
