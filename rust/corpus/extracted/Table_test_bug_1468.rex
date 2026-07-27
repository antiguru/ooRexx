/* extracted from Table::test_bug_1468 */
::routine main public
  t = .Table~new
  f1 = .File~new(1)
  f2 = .File~new(1)
  -- f1 and f2 compare true for = and == ..
  self~assertTrue(f1 = f2)
  self~assertTrue(f1 == f2)
  -- .. but compare false for f1~identityHash == f2~identityHash
  self~assertFalse(f1~identityHash == f2~identityHash)
  -- a Table should see them as the same index
  t[f1] = .true
  t[f2] = f2
  self~assertEquals(1, t~items)
  self~assertTrue(t~hasIndex(f1))
  self~assertTrue(t~hasIndex(f2))

  -- same goes for item equality
  self~assertTrue(t~hasItem(f1))
  self~assertTrue(t~hasItem(f2))



/* Test whether both collections contain the same entries.
   returns .true, if the same, .false else
*/
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
