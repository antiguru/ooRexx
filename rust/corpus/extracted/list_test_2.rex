/* extracted from list::test_2 */
::routine main public
   lst = .list~of()
   self~assertEquals(0, lst~items)
   lst1 = .list~of(1)
   self~assertEquals(1, lst1~items)
   lst2 = .list~of(1,1)
   self~assertEquals(2, lst2~items)
   lst3 = .list~of(1,1,1)
   self~assertEquals(3, lst3~items)
   lst4 = .list~of(1,1,1,1,1,1,1,1,1,1,,
                   2,2,2,2,2,2,2,2,2,2,,
                   3,3,3,3,3,3,3,3,3,3,,
                   4,4,4,4,4,4,4,4,4,4,,
                   5,5,5,5,5,5,5,5,5,5,,
                   6,6,6,6,6,6,6,6,6,6,,
                   7,7,7,7,7,7,7,7,7,7,,
                   8,8,8,8,8,8,8,8,8,8,,
                   9,9,9,9,9,9,9,9,9,9,,
                   0,0,0,0,0,0,0,0,0,0)
   self~assertEquals(100, lst4~items)

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
