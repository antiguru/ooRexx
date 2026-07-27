/* extracted from Queue::test_supplier */
::routine main public

    -- Test that the supplier method returns a supplier.
    q = .queue~new
    q~queue( 1 )
    obj = q~supplier
    self~assertEquals(.supplier, obj~class, 'Supplier method should return a supplier object (1)')

    -- Test the returned supplier is correct.
    self~assertEquals(1, obj~index, 'Supplier index must be 1')
    self~assertEquals(1, obj~item, 'Supplier item must be 1')
    count = 0
    do while obj~available
      count = count + 1
      obj~next
    end
    self~assertEquals(1, count, 'Supplier must have exactly 1 index/item')

    -- Test that an empty queue produces an empty supplier.
    q     = .queue~new
    obj   = q~supplier
    count = 0
    self~assertEquals(.supplier, obj~class, 'Supplier method should return a supplier object (2)')
    do while obj~available
      count = count + 1
      obj~next
    end
    self~assertEquals(0, count, 'Supplier must not have any index/item')


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
