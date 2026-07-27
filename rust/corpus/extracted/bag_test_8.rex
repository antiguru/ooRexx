/* extracted from bag::test_8 */
::routine main public
   bag1 = .bag~of("Mike", "David", "Sam")
   bag2 = .bag~of("Linda", "Lynn")
   bag1~putAll(bag2)
   self~assertEquals(5, bag1~items)
   self~assertEquals(1, bag1~hasIndex("Linda"))
   self~assertEquals(1, bag1~hasIndex("Lynn"))
   arr = .array~of("Linda", "Pam")  -- this adds one duplicate index
   bag1~putAll(arr)
   self~assertEquals(7, bag1~items)
   self~assertEquals(1, bag1~hasIndex("Linda"))
   self~assertEquals(1, bag1~hasIndex("Pam"))
   self~assertEquals(0, bag1~hasIndex("Debbie"))

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
