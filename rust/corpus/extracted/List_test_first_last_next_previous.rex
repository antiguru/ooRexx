/* extracted from List::test_first_last_next_previous */
::routine main public
  expose collDir clz bUserDefinedIndex bSingleItem o1 o2

  a=clz~new
  self~assertNull(a~first)
  self~assertNull(a~last)

  a~append("1v")
  self~assertNull(a~next(a~first))
  self~assertNull(a~previous(a~first))

  n=14
  do n
     a~append(random(n)"v")
  end

  indexes = a~allIndexes

  self~assertEquals(indexes[1], a~first)
  self~assertEquals(indexes[15], a~last)
  self~assertEquals(indexes[2], a~next(a~first))
  self~assertEquals(indexes[3], a~next(a~next(a~first)))
  self~assertEquals(indexes[14], a~previous(a~last))
  self~assertEquals(indexes[1], a~previous(a~next(a~first)))

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
