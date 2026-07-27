/* extracted from circularqueue::test_4 */
::routine main public
   que = .circularqueue~of('Mike', 'David', 'Tom')
   que~resize(5)
   self~assertEquals(3, que~items)
   self~assertEquals(.nil, que[4])
   self~assertEquals(.nil, que[5])
   que~push('Tony')
   self~assertEquals(4, que~items)
   self~assertEquals('Tony', que[1])
   self~assertEquals('Mike', que[2])
   self~assertEquals('David', que[3])
   self~assertEquals('Tom', que[4])
   self~assertEquals(.nil, que[5])
   que~push('Bill')
   self~assertEquals(5, que~items)
   self~assertEquals('Bill', que[1])
   self~assertEquals('Tony', que[2])
   self~assertEquals('Mike', que[3])
   self~assertEquals('David', que[4])
   self~assertEquals('Tom', que[5])
   self~assertEquals(.nil, que[6])
   que~push('Linda')
   self~assertEquals(5, que~items)
   self~assertEquals('Linda', que[1])
   self~assertEquals('Bill', que[2])
   self~assertEquals('Tony', que[3])
   self~assertEquals('Mike', que[4])
   self~assertEquals('David', que[5])
   self~assertEquals(.nil, que[6])

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
