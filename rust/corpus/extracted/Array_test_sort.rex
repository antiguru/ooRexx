/* extracted from Array::test_sort */
::routine main public

  a=.array~of
  a1=a~sort
  a2=.array~of
  self~assertTrue(a1~items=a2~items)
  self~assertTrue(a1~size=a2~size)

  a=.array~of(1)
  a1=a~sort
  a2=.array~of(1)
  self~assertTrue(testSeq(a2, a1))

  a=.array~of(1,2)
  a2=.array~of(1,2)
  a1=a~sort
  self~assertTrue(testSeq(a2, a1))

  a=.array~of(1,2,3)
  a2=.array~of(1,2,3)
  a1=a~sort
  self~assertTrue(testSeq(a2, a1))

  a=.array~of(2,1,3)
  a1=a~sort
  a2=.array~of(1,2,3)
  self~assertTrue(testSeq(a1, a2))

  a=.array~of(2,1,3,2)
  a1=a~sort
  a2=.array~of(1,2,2,3)
  self~assertTrue(testSeq(a1, a2))

  a=.array~of(2,1)
  a1=a~sort
  a2=.array~of(1,2)
  self~assertTrue(testSeq(a1, a2))

  -- the mergesort algorithm uses an insersion sort for partitions with fewer than
  -- 8 items, so we need to throw in a few longer variations

  a=.array~of('y', 'o', 'i', 'x', 'u', 'q', 'e', 'l', 'p', 'b', 'g', 't', 'd', 'c', 'z', 'm', 'h', 'v', 'j', 'r', 'a', 'n', 'f', 'w', 'k', 's')
  a1=a~sort
  a2=.array~of('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z')
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
