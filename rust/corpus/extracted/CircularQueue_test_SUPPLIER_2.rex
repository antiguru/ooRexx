/* extracted from CircularQueue::test_SUPPLIER_2 */
::routine main public
   u0=.CircularQueue~of
   self~assertEquals(0, countSupplierItems(u0~supplier))

   u3=.CircularQueue~of(1,2,3)
   m=u3~SUPPLIER
   self~assertEquals(3, countSupplierItems(m))
   self~assertTrue(testSequence(m, .array~of(1,2,3)))
   -- self~assertTrue(testSequence(m, .array~of(1,2,3)))

   m=u3~SUPPLIER("FIFO")
   self~assertEquals(3, countSupplierItems(m))
   -- self~assertEquals(3, countSupplierItems(u3~supplier("FIFO")))
   self~assertTrue(testSequence(m, .array~of(1,2,3)))

   m=u3~SUPPLIER
   self~assertEquals(3, countSupplierItems(m))
   self~assertFalse(testSequence(m, .array~of(3,2,1)))

   m=u3~SUPPLIER("LIFO")
   self~assertEquals(3, countSupplierItems(m))
   self~assertFalse(testSequence(m, .array~of(1,5,3)))

   m=u3~SUPPLIER("LIFO")
   self~assertEquals(3, countSupplierItems(m))
   self~assertTrue(testSequence(m, .array~of(3,2,1)))
   return


testSequence: procedure
   use arg s, a
   bSame=.true
   i=0
   do while s~available & bSame
      i=i+1
      bSame=(bSame & (s~item=a[i]))
      s~next
   end
   return bSame

countSupplierItems: procedure
   use arg s

   i=0
   s=s~copy    -- work on the copy, otherwise supplier gets "exhausted" by this loop
   do while s~available
      i=i+1
      s~next
   end
   return i


   -- test STRING method ---------------------------------------------
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
