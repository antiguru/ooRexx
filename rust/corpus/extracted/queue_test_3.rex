/* extracted from queue::test_3 */
::routine main public
   que = .queue~of('Mike', 'David', 'Tom')
   que~append('Dick')
   self~assertEquals(4, que~items)
   self~assertEquals('Dick', que[4])
   que~queue('Frank')
   self~assertEquals(5, que~items)
   self~assertEquals('Frank', que[5])
   que~push('Linda')
   self~assertEquals(6, que~items)
   self~assertEquals('Linda', que[1])
   self~assertEquals('Mike', que[2])
   item = que~pull
   self~assertEquals(5, que~items)
   self~assertEquals('Linda', item)
   self~assertEquals('Mike', que[1])
   item = que~remove(1)
   self~assertEquals(4, que~items)
   self~assertEquals('Mike', item)
   item = que~remove(2)
   self~assertEquals(3, que~items)
   self~assertEquals('Tom', item)

   item = que~remove(4)
   self~assertEquals(3, que~items)
   self~assertNull(item)

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
