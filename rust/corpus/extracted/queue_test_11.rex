/* extracted from queue::test_11 */
::routine main public
   que = .queue~of('Mike', 'David', 'Tom')
   idx = que~insert('Frank', .nil)
   self~assertEquals(4, que~items)
   self~assertEquals(1, idx)
   self~assertEquals('Frank', que[1])
   self~assertEquals('Mike', que[2])
   idx = que~insert('Linda', 2)
   self~assertEquals(3, idx)
   self~assertEquals(5, que~items)
   self~assertEquals('Frank', que[1])
   self~assertEquals('Mike', que[2])
   self~assertEquals('Linda', que[3])
   self~assertEquals('David', que[4])
   idx = que~insert('Karen', 5)
   self~assertEquals(6, idx)
   self~assertEquals(6, que~items)
   self~assertEquals('Karen', que[6])

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
