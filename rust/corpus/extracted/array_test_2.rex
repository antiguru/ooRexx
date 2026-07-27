/* extracted from array::test_2 */
::routine main public
   arr = .array~of()
   self~assertEquals(0, arr~items)
   arr1 = .array~of(1)
   self~assertEquals(1, arr1~items)
   arr2 = .array~of(1,1)
   self~assertEquals(2, arr2~items)
   arr3 = .array~of(1,1,1)
   self~assertEquals(3, arr3~items)
   arr4 = .array~of(1,1,1,1,1,1,1,1,1,1,,
                    2,2,2,2,2,2,2,2,2,2,,
                    3,3,3,3,3,3,3,3,3,3,,
                    4,4,4,4,4,4,4,4,4,4,,
                    5,5,5,5,5,5,5,5,5,5,,
                    6,6,6,6,6,6,6,6,6,6,,
                    7,7,7,7,7,7,7,7,7,7,,
                    8,8,8,8,8,8,8,8,8,8,,
                    9,9,9,9,9,9,9,9,9,9,,
                    0,0,0,0,0,0,0,0,0,0)
   self~assertEquals(100, arr4~items)

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
