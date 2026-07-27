/* extracted from CALL::test_14 */
::routine main public
   call on failure
   caught = .false
   call myproc
   if \caught then self~assertTrue(0)
   return

   failure:
   self~assertSame(condition('cs'), 'FAILURE')
   self~assertSame(condition('i'), 'CALL')
   self~assertSame(condition('s'), 'DELAY')
   caught = .true
   return

   myproc:
   raise failure 44.1 return
   return


-- test that an external call correctly searches the requested file
-- according to the documented external search order
-- for a file without extension, the expected search order is
-- 1) parent extension, 2) .REX/.rex, 3) without extension
-- NOTE: although ::REQUIRES Will search .cls first, CALL should not
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
