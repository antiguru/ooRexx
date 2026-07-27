/* extracted from SysStemSort::test_descending_simple */
::routine main public

  -- Expected values
  e.1 = 'f'
  e.2 = 'e'
  e.3 = 'd'
  e.4 = 'c'
  e.5 = 'b'
  e.6 = 'a'

  -- Values to sort
  s.1 = 'c'
  s.2 = 'b'
  s.3 = 'e'
  s.4 = 'a'
  s.5 = 'd'
  s.6 = 'f'
  s.0 = 6

  ret = SysStemSort(s., 'D')
  self~assertTrue(ret == 0)

  -- This tests the stem values are in the expected order
  do i = 1 to s.0
    self~assertEquals(e.i, s.i)
  end

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
