/* extracted from Directory::test_equivalent */
::routine main public
  emptydirectory = .directory~new
  directory1 = .directory~new
  directory1[1] = "a"
  directory1[2] = "b"
  directory1[3] = "c"

  directory2 = directory1~copy

  -- empty directorys of two different sizes...still equivalent


  self~assertTrue(emptydirectory~equivalent(.directory~new))
  -- empty vs. non-empty, two ways
  self~assertFalse(directory1~equivalent(emptydirectory))
  self~assertFalse(emptydirectory~equivalent(directory1))
  -- simple true test
  self~assertTrue(directory1~equivalent(directory2))
  directory2[2] = 'z'
  -- same number of items, different values
  self~assertFalse(directory1~equivalent(directory2))
  -- same values, different indexes
  directory3 = directory1~copy
  directory3~remove(2)
  directory3[4] = "b"
  self~assertFalse(directory1~equivalent(directory3))
  -- different number of items
  directory3~remove(4)
  self~assertFalse(directory1~equivalent(directory3))
  self~assertFalse(directory3~equivalent(directory1))



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
