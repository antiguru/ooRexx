/* extracted from list::test_17 */
::routine main public
   lst = .list~of(1,2,3)
   sidx = lst~first
   newlst = lst~section(sidx)
   self~assertEquals(3, newlst~items)
   self~assertEquals(1, newlst~firstItem)
   self~assertEquals(3, newlst~lastItem)
   newlst = lst~section(sidx, 2)
   self~assertEquals(2, newlst~items)
   self~assertEquals(1, newlst~firstItem)
   self~assertEquals(2, newlst~lastItem)
   sidx = lst~next(sidx)
   newlst = lst~section(sidx)
   self~assertEquals(2, newlst~items)
   self~assertEquals(2, newlst~firstItem)
   self~assertEquals(3, newlst~lastItem)
   newlst = lst~section(sidx, 2)
   self~assertEquals(2, newlst~items)
   self~assertEquals(2, newlst~firstItem)
   self~assertEquals(3, newlst~lastItem)

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
