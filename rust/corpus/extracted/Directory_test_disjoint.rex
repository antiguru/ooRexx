/* extracted from Directory::test_disjoint */
::routine main public
  emptydirectory = .directory~new
  directory1 = .directory~new
  directory1[1] = "a"
  directory1[2] = "b"
  directory1[3] = "c"

  directory2 = directory1~copy
  -- same indexes, different values
  directory3 = .directory~new
  directory1[1] = "x"
  directory1[2] = "y"
  directory1[3] = "z"

  -- empty directorys of two different sizes...still equivalent


  self~assertTrue(emptydirectory~disjoint(.directory~new))
  -- empty vs. non-empty, two ways
  self~assertTrue(directory1~disjoint(emptydirectory))
  self~assertTrue(emptydirectory~disjoint(directory1))
  -- simple overlap test
  self~assertFalse(directory1~disjoint(directory2))
  -- same indices, different values
  self~assertTrue(directory1~disjoint(directory3))
  -- same values, different indexes
  directory3~empty
  directory3[4] = "a"
  directory3[5] = "b"
  directory3[6] = "c"
  self~assertTrue(directory1~disjoint(directory3))
  -- different number of items
  directory2~remove(3)
  self~assertFalse(directory1~disjoint(directory2))


/* --------------------- Directory specific methods -------------------- */

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
