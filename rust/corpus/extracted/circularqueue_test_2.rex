/* extracted from circularqueue::test_2 */
::routine main public
   que = .circularqueue~of('Mike', 'David', 'Tom')
   arr = que~makeArray()
   self~assertEquals(3, arr~items)
   self~assertEquals('Mike', arr[1])
   self~assertEquals('David', arr[2])
   self~assertEquals('Tom', arr[3])
   arr = que~makeArray('F')  -- FIFO
   self~assertEquals(3, arr~items)
   self~assertEquals('Mike', arr[1])
   self~assertEquals('David', arr[2])
   self~assertEquals('Tom', arr[3])
   arr = que~makeArray('L')  -- LIFO
   self~assertEquals(3, arr~items)
   self~assertEquals('Tom', arr[1])
   self~assertEquals('David', arr[2])
   self~assertEquals('Mike', arr[3])

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
