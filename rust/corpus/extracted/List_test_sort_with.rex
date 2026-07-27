/* extracted from List::test_sort_with */
::routine main public

  c=.DescendingComparator~new    -- sort descendingly
  a=.list~of
  a2=.list~of
  a1=a~sortWith(c)
  self~assertTrue(a1~items=a2~items)

  a=.list~of(1)
  a2=.list~of(1)
  a1=a~sortWith(c)
  self~assertTrue(testSeq(a2, a1))

  a=.list~of(1,2)
  a1=a~sortWith(c)
  a2=.list~of(2,1)
  self~assertTrue(testSeq(a1, a2))

  a=.list~of(1,2,3)
  a1=a~sortWith(c)
  a2=.list~of(3,2,1)
  self~assertTrue(testSeq(a1, a2))

  a=.list~of(2,1,3)
  a1=a~sortWith(c)
  a2=.list~of(3,2,1)
  self~assertTrue(testSeq(a1, a2))


  a=.list~of(2,1,3,2)
  a1=a~sortWith(c)
  a2=.list~of(3,2,2,1)
  self~assertTrue(testSeq(a1, a2))

  a=.list~of(2,1)
  a1=a~sortWith(c)
  a2=.list~of(2,1)
  self~assertTrue(testSeq(a1, a2))

  -- the mergesort algorithm uses an insersion sort for partitions with fewer than
  -- 8 items, so we need to throw in a few longer variations

  a=.list~of('y', 'o', 'i', 'x', 'u', 'q', 'e', 'l', 'p', 'b', 'g', 't', 'd', 'c', 'z', 'm', 'h', 'v', 'j', 'r', 'a', 'n', 'f', 'w', 'k', 's')
  a1=a~sortWith(c)
  a2=.list~of('z', 'y', 'x', 'w', 'v', 'u', 't', 's', 'r', 'q', 'p', 'o', 'n', 'm', 'l', 'k', 'j', 'i', 'h', 'g', 'f', 'e', 'd', 'c', 'b', 'a')
  self~assertTrue(testSeq(a1, a2))

  return


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
