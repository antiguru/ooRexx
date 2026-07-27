/* extracted from SysStemSort::test_compound_name */
::routine main public

  -- Set up out expected values
  e.1 = 'a'
  e.2 = 'b'
  e.3 = 'c'
  e.4 = 'd'
  e.5 = 'e'
  e.6 = 'f'

  -- Set up the values to sort
  s.foo.1 = 'c'
  s.foo.2 = 'b'
  s.foo.3 = 'a'
  s.foo.4 = 'f'
  s.foo.5 = 'd'
  s.foo.6 = 'e'
  s.foo.0 = 6

  -- Do the sort.
  ret = SysStemSort('s.foo')

  -- This tests that the return is equal to 0
  self~assertTrue(ret == 0)

  -- This tests the stem values are in the expected order
  do i = 1 to s.foo.0
    self~assertEquals(e.i, s.foo.i)
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
